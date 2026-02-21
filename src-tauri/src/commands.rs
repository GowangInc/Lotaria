use crate::capture::ScreenCapture;
use crate::state::{get_interval_seconds, truncate_response, Config, History, ProviderDef, StateManager};
use crate::tts::{self, create_tts_service};
use crate::vision::create_vision_service;
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
        "vision_model" => config.vision_model = value.as_str().unwrap_or("gemini-2.0-flash").to_string(),
        "tts_provider" => config.tts_provider = value.as_str().unwrap_or("gemini").to_string(),
        "tts_model" => config.tts_model = value.as_str().unwrap_or("gemini-2.5-flash-live").to_string(),
        "tts_voice" => config.tts_voice = value.as_str().unwrap_or("Kore").to_string(),
        "speech_bubble_enabled" => config.speech_bubble_enabled = value.as_bool().unwrap_or(true),
        "audio_enabled" => config.audio_enabled = value.as_bool().unwrap_or(true),
        "mood" => config.mood = value.as_str().unwrap_or("roast").to_string(),
        "pet_style" => config.pet_style = value.as_str().unwrap_or("default").to_string(),
        "gemini_free_tier" => config.gemini_free_tier = value.as_bool().unwrap_or(true),
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
    let config = state.config.read().await.clone();

    // Check if we have API keys
    let vision_provider_def = ProviderDef::get(&config.vision_provider)
        .ok_or_else(|| "Invalid vision provider".to_string())?;

    let vision_api_key = config.api_keys.get(&config.vision_provider)
        .cloned()
        .or_else(|| std::env::var(&vision_provider_def.env_var).ok())
        .ok_or_else(|| "Vision API key not set".to_string())?;

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

    // Build prompt
    let history = state.history.read().await;
    let prompt = state.state_manager.build_prompt(&config.mood, &history);
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

    // Add to history
    let mut history = state.history.write().await;
    state.state_manager.add_to_history(&analysis, timestamp, &mut history)
        .map_err(|e| e.to_string())?;
    drop(history);

    // TTS
    let mut audio_base64 = None;
    let mut audio_duration = 0.0;

    if config.audio_enabled {
        let tts_provider_def = ProviderDef::get(&config.tts_provider)
            .ok_or_else(|| "Invalid TTS provider".to_string())?;

        let tts_api_key = if config.tts_provider == config.vision_provider {
            config.api_keys.get(&config.tts_provider)
                .cloned()
                .or_else(|| std::env::var(&tts_provider_def.env_var).ok())
        } else {
            config.api_keys.get(&config.tts_provider).cloned()
        }.ok_or_else(|| "TTS API key not set".to_string())?;

        let tts_service = create_tts_service(
            &config.tts_provider,
            tts_api_key,
            config.tts_model.clone(),
            config.tts_voice.clone()
        );

        match tts_service.synthesize(&analysis).await {
            Ok(audio_bytes) => {
                let word_count = analysis.split_whitespace().count();
                audio_duration = (word_count as f64 / 150.0) * 60.0;

                let _ = tts::AudioPlayer::play_async(audio_bytes.clone());
                audio_base64 = Some(b64_encode(&audio_bytes));
            }
            Err(e) => {
                tracing::error!("TTS error: {}", e);
            }
        }
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
    loop {
        // Check if monitoring is still enabled
        let is_active = monitoring.lock().map(|g| *g).unwrap_or(false);
        if !is_active {
            tracing::info!("Monitoring loop exiting");
            break;
        }

        // Read current config for interval
        let config = config_lock.read().await;
        let interval_secs = get_interval_seconds(
            &config.interval,
            config.gemini_free_tier,
            &config.tts_provider,
        );
        drop(config);

        sleep(Duration::from_secs(interval_secs)).await;

        // Check again after sleep
        let is_active = monitoring.lock().map(|g| *g).unwrap_or(false);
        if !is_active {
            break;
        }

        // Emit event for frontend to trigger roast
        let _ = app_handle.emit("monitoring-tick", ());
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

/// Quit the app
#[tauri::command]
pub fn quit(app_handle: AppHandle) {
    tracing::info!("Quitting app");
    app_handle.exit(0);
}

fn b64_encode(input: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(input)
}
