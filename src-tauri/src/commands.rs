use crate::capture::ScreenCapture;
use crate::state::{get_interval_seconds, truncate_response, Config, History, ProviderDef, ScreenContext, StateManager, INTERVAL_PRESETS};
use crate::tts::{self, create_tts_service, SoundEffects};
use crate::vision::create_vision_service;
use chrono::Timelike;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State, Emitter};
use tokio::time::{sleep, Duration};
use tokio::sync::RwLock;

/// App state shared across commands
pub struct AppState {
    pub state_manager: Arc<StateManager>,
    pub config: Arc<RwLock<Config>>,
    pub history: Arc<RwLock<History>>,
    pub monitoring: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new(state_manager: StateManager) -> anyhow::Result<Self> {
        let config = state_manager.load_config()?;
        let history = state_manager.load_history()?;

        Ok(Self {
            state_manager: Arc::new(state_manager),
            config: Arc::new(RwLock::new(config)),
            history: Arc::new(RwLock::new(history)),
            monitoring: Arc::new(Mutex::new(false)),
        })
    }
}

/// Roast result sent to frontend
#[derive(Serialize, Clone)]
pub struct RoastResult {
    pub text: String,
    pub audio_base64: Option<String>,
    pub audio_duration: f64,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Get full config (with masked API keys)
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Config, String> {
    let config = state.config.read().await;
    Ok(config.clone())
}

/// Get providers list
#[tauri::command]
pub fn get_providers() -> Vec<ProviderDef> {
    ProviderDef::all()
}

/// Get masked API keys
#[tauri::command]
pub async fn get_api_keys(state: State<'_, AppState>) -> Result<std::collections::HashMap<String, String>, String> {
    let config = state.config.read().await;
    Ok(state.state_manager.get_masked_api_keys(&config))
}

/// Save API key
#[tauri::command]
pub async fn save_api_key(
    provider: String,
    key: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut config = state.config.write().await;
    config.api_keys.insert(provider, key);
    state.state_manager.save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

/// Set config value
#[tauri::command]
pub async fn set_config(
    key: String,
    value: serde_json::Value,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut config = state.config.write().await;

    match key.as_str() {
        "is_active" => config.is_active = value.as_bool().unwrap_or(false),
        "interval" => config.interval = value.as_str().unwrap_or("frequent").to_string(),
        "vision_provider" => config.vision_provider = value.as_str().unwrap_or("gemini").to_string(),
        "vision_model" => config.vision_model = value.as_str().unwrap_or("gemini-2.5-flash").to_string(),
        "tts_provider" => config.tts_provider = value.as_str().unwrap_or("gemini").to_string(),
        "tts_model" => config.tts_model = value.as_str().unwrap_or("gemini-2.5-flash-preview-tts").to_string(),
        "tts_voice" => config.tts_voice = value.as_str().unwrap_or("Kore").to_string(),
        "speech_bubble_enabled" => config.speech_bubble_enabled = value.as_bool().unwrap_or(true),
        "audio_enabled" => config.audio_enabled = value.as_bool().unwrap_or(true),
        "mood" => config.mood = value.as_str().unwrap_or("roast").to_string(),
        "custom_mood" => config.custom_mood = value.as_str().unwrap_or("").to_string(),
        "pet_style" => config.pet_style = value.as_str().unwrap_or("default").to_string(),
        "gemini_free_tier" => config.gemini_free_tier = value.as_bool().unwrap_or(true),
        "roast_intensity" => config.roast_intensity = value.as_u64().unwrap_or(5).min(10) as u8,
        "mood_rotation" => config.mood_rotation = value.as_str().unwrap_or("").to_string(),
        "blacklist" => {
            config.blacklist = value.as_str()
                .unwrap_or("")
                .split('\n')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        "break_reminder_minutes" => config.break_reminder_minutes = value.as_u64().unwrap_or(0).min(240) as u32,
        "first_run" => config.first_run = value.as_bool().unwrap_or(false),
        _ => {}
    }

    state.state_manager.save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

/// Perform a roast (capture + analyze + TTS)
#[tauri::command]
pub async fn roast_now(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>
) -> Result<RoastResult, String> {
    let mut config = state.config.read().await.clone();

    // Apply mood rotation if enabled
    if config.mood_rotation == "per-roast" && config.mood != "custom" {
        use crate::state::MOOD_PROMPTS;
        use rand::Rng;
        let idx = rand::thread_rng().gen_range(0..MOOD_PROMPTS.len());
        config.mood = MOOD_PROMPTS[idx].0.to_string();
        tracing::info!("Mood rotated to: {}", config.mood);
    } else if config.mood_rotation == "scheduled" && config.mood != "custom" {
        let hour = chrono::Local::now().hour();
        let scheduled_mood = match hour {
            6..=10 => "encouraging",   // Morning: cheerful start
            11..=14 => "helpful",      // Midday: productivity focus
            15..=18 => "sarcastic",    // Afternoon: dry wit
            19..=22 => "roast",        // Evening: savage mode
            _ => "zen",               // Late night: philosophical
        };
        config.mood = scheduled_mood.to_string();
        tracing::info!("Scheduled mood for hour {}: {}", hour, scheduled_mood);
    }

    // Check blacklist — skip roast if foreground window matches
    if !config.blacklist.is_empty() {
        if let Some(title) = get_foreground_window_title() {
            let title_lower = title.to_lowercase();
            for entry in &config.blacklist {
                if !entry.is_empty() && title_lower.contains(&entry.to_lowercase()) {
                    tracing::info!("Blacklisted window detected: '{}' matches '{}'", title, entry);
                    return Ok(RoastResult {
                        text: String::new(),
                        audio_base64: None,
                        audio_duration: 0.0,
                        timestamp: chrono::Local::now().timestamp(),
                        error: Some("skipped_blacklist".to_string()),
                    });
                }
            }
        }
    }

    // Check if we have API keys (skip for local providers)
    let vision_provider_def = ProviderDef::get(&config.vision_provider)
        .ok_or_else(|| "Invalid vision provider".to_string())?;

    // Local providers that don't need API keys
    let is_local_vision = config.vision_provider == "ollama";

    let vision_api_key = if is_local_vision {
        String::new() // Local providers don't need API keys
    } else {
        config.api_keys.get(&config.vision_provider)
            .cloned()
            .or_else(|| std::env::var(&vision_provider_def.env_var).ok())
            .ok_or_else(|| "Vision API key not set".to_string())?
    };

    // Play start sound effect
    if config.audio_enabled {
        SoundEffects::play_start();
    }

    // --- Gather screen context BEFORE hiding window ---
    let ctx_foreground_title = get_foreground_window_title().unwrap_or_default();
    let ctx_foreground_process = get_foreground_process_name().unwrap_or_default();
    let ctx_open_windows = enumerate_visible_windows();
    let ctx_idle_seconds = get_system_idle_seconds();

    tracing::info!("Screen context: fg='{}' proc='{}' windows={} idle={}s",
        ctx_foreground_title, ctx_foreground_process, ctx_open_windows.len(), ctx_idle_seconds);

    // Move window off-screen before capture
    let original_pos = window.outer_position().map_err(|e| e.to_string())?;
    window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: -1000, y: -1000 }))
        .map_err(|e| e.to_string())?;

    // Small delay for window move
    sleep(Duration::from_millis(100)).await;

    // Capture screen
    let capture = ScreenCapture::capture_primary(&state.state_manager.temp_dir())
        .map_err(|e| e.to_string())?;

    // Restore window
    window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: original_pos.x, y: original_pos.y }))
        .map_err(|e| e.to_string())?;

    // Compute perceptual hash from captured image
    let screen_hash = crate::capture::compute_average_hash(&capture.png_bytes);

    let screen_context = ScreenContext {
        foreground_title: ctx_foreground_title,
        foreground_process: ctx_foreground_process,
        open_windows: ctx_open_windows,
        idle_seconds: ctx_idle_seconds,
        screen_hash,
        timestamp: chrono::Local::now().timestamp(),
    };

    // Build prompt with context diff
    let history = state.history.read().await;
    let prompt = state.state_manager.build_prompt_with_context(&config, &history, &screen_context);
    drop(history);

    // Analyze with vision service
    let vision_service = create_vision_service(
        &config.vision_provider,
        vision_api_key,
        config.vision_model.clone()
    );

    let analysis = match vision_service.analyze(&capture.base64, &prompt).await {
        Ok(text) => truncate_response(&text, 500),
        Err(e) => {
            return Ok(RoastResult {
                text: format!("Vision analysis failed: {}", e),
                audio_base64: None,
                audio_duration: 0.0,
                timestamp: chrono::Local::now().timestamp(),
                error: Some(e.to_string()),
            });
        }
    };

    let timestamp = chrono::Local::now().timestamp();

    // Add to history with context
    let mut history = state.history.write().await;
    state.state_manager.add_to_history_with_context(&analysis, timestamp, screen_context, &mut history)
        .map_err(|e| e.to_string())?;
    drop(history);

    // TTS
    let mut audio_base64 = None;
    let audio_duration;

    if config.audio_enabled {
        let tts_provider_def = ProviderDef::get(&config.tts_provider)
            .ok_or_else(|| "Invalid TTS provider".to_string())?;

        // Local TTS providers that don't need API keys
        let is_local_tts = config.tts_provider == "piper";

        let tts_api_key = if is_local_tts {
            String::new() // Local TTS doesn't need API keys
        } else if config.tts_provider == config.vision_provider {
            config.api_keys.get(&config.tts_provider)
                .cloned()
                .or_else(|| std::env::var(&tts_provider_def.env_var).ok())
                .ok_or_else(|| "TTS API key not set".to_string())?
        } else {
            config.api_keys.get(&config.tts_provider)
                .cloned()
                .or_else(|| std::env::var(&tts_provider_def.env_var).ok())
                .ok_or_else(|| "TTS API key not set".to_string())?
        };

        let tts_service = create_tts_service(
            &config.tts_provider,
            tts_api_key,
            config.tts_model.clone(),
            config.tts_voice.clone()
        );

        match tts_service.synthesize(&analysis).await {
            Ok(audio_bytes) => {
                // Save audio to file for debugging
                let audio_path = state.state_manager.temp_dir().join(format!("audio_{}.wav", timestamp));
                if let Err(e) = std::fs::write(&audio_path, &audio_bytes) {
                    tracing::error!("Failed to save audio file: {}", e);
                } else {
                    tracing::info!("Audio saved to: {:?}", audio_path);
                }

                let _ = tts::AudioPlayer::play_async(audio_bytes.clone());
                audio_base64 = Some(b64_encode(&audio_bytes));
            }
            Err(e) => {
                tracing::error!("TTS error: {}", e);
            }
        }
    }

    // Always compute display duration from text, regardless of TTS success
    let word_count = analysis.split_whitespace().count();
    audio_duration = (word_count as f64 / 150.0) * 60.0;

    // Play completion chime (after TTS so they don't overlap)
    if config.audio_enabled {
        SoundEffects::play_complete();
    }

    Ok(RoastResult {
        text: analysis,
        audio_base64,
        audio_duration,
        timestamp,
        error: None,
    })
}

/// Toggle monitoring on/off
#[tauri::command]
pub async fn toggle_monitoring(
    app_handle: AppHandle,
    state: State<'_, AppState>
) -> Result<bool, String> {
    let is_monitoring = *state.monitoring.lock().map_err(|e| e.to_string())?;

    if is_monitoring {
        // Stop monitoring
        *state.monitoring.lock().map_err(|e| e.to_string())? = false;

        let mut config = state.config.write().await;
        config.is_active = false;
        state.state_manager.save_config(&config).map_err(|e| e.to_string())?;

        tracing::info!("Monitoring stopped");
        Ok(false)
    } else {
        // Start monitoring
        *state.monitoring.lock().map_err(|e| e.to_string())? = true;

        {
            let mut cfg = state.config.write().await;
            cfg.is_active = true;
            state.state_manager.save_config(&cfg).map_err(|e| e.to_string())?;
        }

        // Clone Arcs for the spawned task
        let monitoring = state.monitoring.clone();
        let config_lock = state.config.clone();
        let app_handle_clone = app_handle.clone();

        tokio::spawn(async move {
            monitoring_loop(app_handle_clone, monitoring, config_lock).await;
        });

        tracing::info!("Monitoring started");
        Ok(true)
    }
}

/// Background monitoring loop
async fn monitoring_loop(
    app_handle: AppHandle,
    monitoring: Arc<Mutex<bool>>,
    config_lock: Arc<RwLock<Config>>,
) {
    let mut elapsed_secs = 0;
    let mut break_elapsed_secs: u64 = 0;

    // Calculate initial interval
    let config = config_lock.read().await;
    let mut interval_secs = get_interval_seconds(
        &config.interval,
        config.gemini_free_tier,
        &config.tts_provider,
    );
    drop(config);

    tracing::info!("Monitoring loop started with interval: {} seconds", interval_secs);

    loop {
        // Check if monitoring is still enabled
        let is_active = monitoring.lock().map(|g| *g).unwrap_or(false);
        if !is_active {
            tracing::info!("Monitoring loop exiting");
            break;
        }

        if elapsed_secs >= interval_secs {
            elapsed_secs = 0;

            // Emit event for frontend to trigger roast
            let _ = app_handle.emit("monitoring-tick", ());

            // Calculate next interval after roast
            let config = config_lock.read().await;
            interval_secs = get_interval_seconds(
                &config.interval,
                config.gemini_free_tier,
                &config.tts_provider,
            );
            drop(config);

            tracing::info!("Next roast scheduled in {} seconds", interval_secs);
        }

        // Break reminder check
        let config = config_lock.read().await;
        let break_mins = config.break_reminder_minutes;
        drop(config);

        if break_mins > 0 {
            let break_secs = (break_mins as u64) * 60;
            if break_elapsed_secs >= break_secs {
                break_elapsed_secs = 0;
                tracing::info!("Break reminder triggered after {} minutes", break_mins);
                let _ = app_handle.emit("break-reminder", ());
            }
        }

        sleep(Duration::from_secs(1)).await;
        elapsed_secs += 1;
        break_elapsed_secs += 1;
    }
}

/// Clear history
#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let mut history = state.history.write().await;
    state.state_manager.clear_history(&mut history)
        .map_err(|e| e.to_string())
}

/// Get history
#[tauri::command]
pub async fn get_history(state: State<'_, AppState>) -> Result<History, String> {
    let history = state.history.read().await;
    Ok(history.clone())
}

/// Mark first run complete
#[tauri::command]
pub async fn mark_first_run_complete(state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.write().await;
    config.first_run = false;
    state.state_manager.save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

/// Get moods
#[tauri::command]
pub fn get_moods() -> Vec<(String, String)> {
    crate::state::MOOD_PROMPTS
        .iter()
        .map(|(k, _)| (k.to_string(), k.chars().next().unwrap().to_uppercase().collect::<String>() + &k[1..]))
        .collect()
}

/// Get interval presets
#[tauri::command]
pub fn get_intervals() -> Vec<(String, String)> {
    INTERVAL_PRESETS
        .iter()
        .map(|(key, (_min, _max))| {
            let label = match *key {
                "often" => "Often (5-10 min)",
                "frequent" => "Frequent (10-20 min)",
                "infrequent" => "Infrequent (25-45 min)",
                _ => key,
            };
            (key.to_string(), label.to_string())
        })
        .collect()
}

/// Improve custom mood with AI
#[tauri::command]
pub async fn improve_mood(
    mood_text: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let config = state.config.read().await.clone();

    // Get vision API key (skip for local providers)
    let vision_provider_def = ProviderDef::get(&config.vision_provider)
        .ok_or_else(|| "Invalid vision provider".to_string())?;

    let is_local_vision = config.vision_provider == "ollama";

    let vision_api_key = if is_local_vision {
        String::new() // Local providers don't need API keys
    } else {
        config.api_keys.get(&config.vision_provider)
            .cloned()
            .or_else(|| std::env::var(&vision_provider_def.env_var).ok())
            .ok_or_else(|| "Vision API key not set".to_string())?
    };

    // Create vision service
    let vision_service = create_vision_service(
        &config.vision_provider,
        vision_api_key,
        config.vision_model.clone()
    );

    // Build improvement prompt
    let improvement_prompt = format!(
        r#"You are an expert at writing system prompts for AI assistants. The user has written this custom mood/personality prompt for a desktop pet that roasts them:

"{}"

Your task: Improve this prompt to make it more effective, specific, and entertaining. Follow these guidelines:
- Make it clear, actionable, and specific about the desired tone and behavior
- Add constraints (character limits, format requirements, etc.) if missing
- Ensure it instructs the AI to analyze the FULL context (apps, time, tabs, etc.)
- Make it more vivid and personality-driven
- Keep the core intent but enhance the execution
- Keep it under 500 characters for the final output

Return ONLY the improved prompt text, no explanations or meta-commentary."#,
        mood_text
    );

    // Call vision API (no image needed for text improvement)
    match vision_service.analyze("", &improvement_prompt).await {
        Ok(improved) => Ok(truncate_response(&improved, 800)),
        Err(e) => Err(format!("Failed to improve mood: {}", e)),
    }
}

/// Quit the app
#[tauri::command]
pub fn quit(app_handle: AppHandle) {
    tracing::info!("Quitting app");
    app_handle.exit(0);
}

/// Get global cursor position (Windows FFI)
#[tauri::command]
pub fn get_cursor_position() -> Result<(i32, i32), String> {
    #[cfg(target_os = "windows")]
    {
        use std::mem::MaybeUninit;

        #[repr(C)]
        struct POINT {
            x: i32,
            y: i32,
        }

        extern "system" {
            fn GetCursorPos(lpPoint: *mut POINT) -> i32;
        }

        let mut point = MaybeUninit::<POINT>::uninit();
        let result = unsafe { GetCursorPos(point.as_mut_ptr()) };
        if result != 0 {
            let point = unsafe { point.assume_init() };
            Ok((point.x, point.y))
        } else {
            Err("GetCursorPos failed".to_string())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("get_cursor_position is only supported on Windows".to_string())
    }
}

/// Get Windows accent color via DwmGetColorizationColor
#[tauri::command]
pub fn get_accent_color() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn DwmGetColorizationColor(pcrColorization: *mut u32, pfOpaqueBlend: *mut i32) -> i32;
        }

        let mut color: u32 = 0;
        let mut opaque: i32 = 0;
        let hr = unsafe { DwmGetColorizationColor(&mut color, &mut opaque) };
        if hr >= 0 {
            let r = (color >> 16) & 0xFF;
            let g = (color >> 8) & 0xFF;
            let b = color & 0xFF;
            Ok(format!("#{:02x}{:02x}{:02x}", r, g, b))
        } else {
            Err("DwmGetColorizationColor failed".to_string())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("#e94560".to_string())
    }
}

/// Set whether the window should ignore cursor events (click-through)
#[tauri::command]
pub fn set_ignore_cursor_events(window: tauri::WebviewWindow, ignore: bool) -> Result<(), String> {
    tracing::info!("Setting ignore_cursor_events to: {}", ignore);
    window.set_ignore_cursor_events(ignore).map_err(|e| e.to_string())
}

/// Get the foreground window title (for blacklist checking)
fn get_foreground_window_title() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn GetForegroundWindow() -> isize;
            fn GetWindowTextW(hWnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
        }

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return None;
            }
            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            if len > 0 {
                Some(String::from_utf16_lossy(&buf[..len as usize]))
            } else {
                None
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// Get the foreground window's process name (e.g. "Code.exe", "chrome.exe")
fn get_foreground_process_name() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn GetForegroundWindow() -> isize;
            fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
            fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
            fn CloseHandle(hObject: isize) -> i32;
            fn QueryFullProcessImageNameW(
                hProcess: isize, dwFlags: u32,
                lpExeName: *mut u16, lpdwSize: *mut u32,
            ) -> i32;
        }
        const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 { return None; }

            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 { return None; }

            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle == 0 { return None; }

            let mut buf = [0u16; 512];
            let mut size = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
            CloseHandle(handle);

            if ok != 0 && size > 0 {
                let full_path = String::from_utf16_lossy(&buf[..size as usize]);
                full_path.rsplit('\\').next().map(|s| s.to_string())
            } else {
                None
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    { None }
}

/// Enumerate all visible windows with non-empty titles
fn enumerate_visible_windows() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn EnumWindows(lpEnumFunc: extern "system" fn(isize, isize) -> i32, lParam: isize) -> i32;
            fn IsWindowVisible(hWnd: isize) -> i32;
            fn GetWindowTextW(hWnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
            fn GetWindowTextLengthW(hWnd: isize) -> i32;
        }

        use std::sync::Mutex as StdMutex;
        static RESULTS: StdMutex<Vec<String>> = StdMutex::new(Vec::new());

        extern "system" fn enum_callback(hwnd: isize, _: isize) -> i32 {
            unsafe {
                if IsWindowVisible(hwnd) == 0 { return 1; }
                let len = GetWindowTextLengthW(hwnd);
                if len <= 0 { return 1; }

                let mut buf = vec![0u16; (len + 1) as usize];
                let actual = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
                if actual > 0 {
                    let title = String::from_utf16_lossy(&buf[..actual as usize]);
                    if title.len() > 1
                        && title != "Program Manager"
                        && !title.starts_with("MSCTFIME")
                    {
                        if let Ok(mut results) = RESULTS.lock() {
                            results.push(title);
                        }
                    }
                }
            }
            1
        }

        if let Ok(mut r) = RESULTS.lock() { r.clear(); }
        unsafe { EnumWindows(enum_callback, 0); }
        RESULTS.lock().map(|r| r.clone()).unwrap_or_default()
    }

    #[cfg(not(target_os = "windows"))]
    { Vec::new() }
}

/// Get seconds since last user input (mouse/keyboard)
fn get_system_idle_seconds() -> u64 {
    #[cfg(target_os = "windows")]
    {
        #[repr(C)]
        struct LASTINPUTINFO {
            cb_size: u32,
            dw_time: u32,
        }
        extern "system" {
            fn GetLastInputInfo(plii: *mut LASTINPUTINFO) -> i32;
            fn GetTickCount() -> u32;
        }

        unsafe {
            let mut lii = LASTINPUTINFO { cb_size: 8, dw_time: 0 };
            if GetLastInputInfo(&mut lii) != 0 {
                let now = GetTickCount();
                let elapsed_ms = now.wrapping_sub(lii.dw_time);
                (elapsed_ms / 1000) as u64
            } else {
                0
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    { 0 }
}

/// Check if Ollama is running and fetch available models
#[tauri::command]
pub async fn get_ollama_models() -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();

    // Check if Ollama is running
    let response = client
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map_err(|e| format!("Ollama not running: {}", e))?;

    if !response.status().is_success() {
        return Err("Ollama API error".to_string());
    }

    #[derive(serde::Deserialize)]
    struct OllamaModel {
        name: String,
    }

    #[derive(serde::Deserialize)]
    struct OllamaResponse {
        models: Vec<OllamaModel>,
    }

    let ollama_response: OllamaResponse = response.json().await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    // Filter for vision models - known vision-capable model families
    let vision_models: Vec<String> = ollama_response.models
        .into_iter()
        .filter(|m| {
            let name_lower = m.name.to_lowercase();
            // Common vision model families
            name_lower.contains("vision") ||      // llama3.2-vision, qwen-vl, etc.
            name_lower.contains("llava") ||       // llava, bakllava
            name_lower.contains("minicpm") ||     // minicpm-v
            name_lower.contains("moondream") ||   // moondream2
            name_lower.contains("qwen") && name_lower.contains("vl") ||  // qwen3-vl, qwen2-vl
            name_lower.contains("phi") && name_lower.contains("vision") || // phi-3-vision
            name_lower.contains("llama4") ||      // llama4 is natively multimodal
            name_lower.contains("cogvlm") ||      // cogvlm
            name_lower.contains("internvl") ||    // internvl
            name_lower.contains("yi-vl")          // yi-vl
        })
        .map(|m| m.name)
        .collect();

    Ok(vision_models)
}

fn b64_encode(input: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(input)
}
