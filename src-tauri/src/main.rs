#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lotaria::{commands::*, state::StateManager};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, Emitter, window::Color};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;

fn main() {
    // Set up panic hook to see errors
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {:?}\n", info);
        let _ = std::fs::write("lotaria_panic.log", &msg);
        eprintln!("{}", msg);
        tracing::error!("{}", msg);
    }));

    // Initialize tracing with rotating file logging
    let log_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("lotaria");

    // Create log directory if it doesn't exist
    let _ = std::fs::create_dir_all(&log_dir);

    // Daily rotation: keeps last 7 days of logs, max 10MB per file
    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");

    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_tts::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            tracing::info!("=== Lotaria Starting ===");
            
            // Initialize state manager
            let state_manager = match StateManager::new() {
                Ok(sm) => sm,
                Err(e) => {
                    tracing::error!("Failed to create StateManager: {}", e);
                    return Err(e.to_string().into());
                }
            };
            
            let app_state = match AppState::new(state_manager) {
                Ok(state) => state,
                Err(e) => {
                    tracing::error!("Failed to create AppState: {}", e);
                    return Err(e.to_string().into());
                }
            };

            // Get initial config values BEFORE managing state
            let is_first_run = app_state.config.blocking_read().first_run;

            // Clone the Arcs we need for monitoring BEFORE managing state
            let monitoring_for_spawn = app_state.monitoring.clone();
            let config_lock_for_spawn = app_state.config.clone();

            // Now manage the state
            app.manage(app_state);
            tracing::info!("AppState managed");

            // Create main window - start visible immediately
            let _window = match WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Lotaria")
                .inner_size(420.0, 400.0)
                .decorations(false)
                .transparent(true)
                .shadow(false)
                .always_on_top(true)
                .resizable(true)
                .skip_taskbar(false)
                .visible(true)
                .center()
                .background_color(Color(0, 0, 0, 0))
                .build()
            {
                Ok(w) => {
                    tracing::info!("Window created successfully");
                    // Enable click-through by default (clicks pass to windows underneath)
                    if let Err(e) = w.set_ignore_cursor_events(true) {
                        tracing::warn!("Failed to set ignore cursor events: {}", e);
                    } else {
                        tracing::info!("Click-through enabled");
                    }
                    w
                }
                Err(e) => {
                    tracing::error!("Failed to create window: {}", e);
                    return Err(e.to_string().into());
                }
            };

            tracing::info!("Window created");

            // System tray
            let roast_item = MenuItemBuilder::with_id("roast", "🔥 Roast Now").build(app)?;
            let settings_item = MenuItemBuilder::with_id("settings", "⚙️ Settings").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .item(&roast_item)
                .separator()
                .item(&settings_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(tauri::image::Image::from_path("icons/32x32.png").unwrap_or_else(|_| {
                    tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")).expect("bundled icon")
                }))
                .menu(&tray_menu)
                .tooltip("Lotaria")
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "roast" => {
                            let _ = app.emit("monitoring-tick", ());
                        }
                        "settings" => {
                            let _ = app.emit("tray-open-settings", ());
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            tracing::info!("System tray created");

            // Always start monitoring (after first run setup)
            if !is_first_run {
                tracing::info!("Auto-starting monitoring...");
                *monitoring_for_spawn.lock().unwrap() = true;

                // Save is_active = true
                {
                    let mut cfg = config_lock_for_spawn.blocking_write();
                    cfg.is_active = true;
                    let sm_ref = app.state::<AppState>();
                    let _ = sm_ref.state_manager.save_config(&cfg);
                }

                let app_handle = app.handle().clone();
                let mon = monitoring_for_spawn.clone();
                let cfg_lock = config_lock_for_spawn.clone();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        use tokio::time::{sleep, Duration};
                        use lotaria::state::get_interval_seconds;

                        let mut elapsed_secs: u64 = 0;
                        let mut interval_secs = {
                            let config = cfg_lock.read().await;
                            get_interval_seconds(&config.interval, config.gemini_free_tier, &config.tts_provider)
                        };
                        tracing::info!("Monitoring loop started, interval: {}s", interval_secs);

                        loop {
                            let active = mon.lock().map(|g| *g).unwrap_or(false);
                            if !active { tracing::info!("Monitoring stopped"); break; }

                            if elapsed_secs >= interval_secs {
                                tracing::info!("Triggering roast after {}s", elapsed_secs);
                                elapsed_secs = 0;
                                let _ = app_handle.emit("monitoring-tick", ());

                                interval_secs = {
                                    let config = cfg_lock.read().await;
                                    get_interval_seconds(&config.interval, config.gemini_free_tier, &config.tts_provider)
                                };
                                tracing::info!("Next interval: {}s", interval_secs);
                            }

                            sleep(Duration::from_secs(1)).await;
                            elapsed_secs += 1;
                        }
                    });
                });

                tracing::info!("Monitoring thread spawned");
            }

            // Clean up old files on startup
            {
                let state = app.state::<AppState>();
                let _ = state.state_manager.cleanup_old_files();
            }

            tracing::info!("=== Setup Complete ===");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_providers,
            get_api_keys,
            save_api_key,
            set_config,
            roast_now,
            toggle_monitoring,
            clear_history,
            get_history,
            mark_first_run_complete,
            get_moods,
            get_intervals,
            improve_mood,
            quit,
            get_cursor_position,
            get_accent_color,
            set_ignore_cursor_events,
            get_ollama_models,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
