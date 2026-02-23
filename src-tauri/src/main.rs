#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lotaria::{commands::*, state::StateManager};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, Emitter, window::Color};

fn main() {
    // Set up panic hook to see errors
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {:?}\n", info);
        let _ = std::fs::write("lotaria_panic.log", &msg);
        eprintln!("{}", msg);
        tracing::error!("{}", msg);
    }));
    
    // Initialize tracing with file logging
    let log_path = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("lotaria")
        .join("app.log");
    
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap_or_else(|_| std::fs::File::create("lotaria.log").unwrap());
    
    tracing_subscriber::fmt()
        .with_writer(move || log_file.try_clone().unwrap())
        .with_ansi(false)
        .init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
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
            let is_active = app_state.config.blocking_read().is_active;
            
            tracing::info!("First run: {}, Is active: {}", is_first_run, is_active);

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
                    w
                }
                Err(e) => {
                    tracing::error!("Failed to create window: {}", e);
                    return Err(e.to_string().into());
                }
            };

            tracing::info!("Window created");

            // Auto-start monitoring if not first run and was previously active
            if !is_first_run && is_active {
                tracing::info!("Auto-starting monitoring...");
                *monitoring_for_spawn.lock().unwrap() = true;
                let app_handle = app.handle().clone();
                
                // Spawn monitoring in a new thread to avoid blocking
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        super_monitoring_loop(app_handle, monitoring_for_spawn, config_lock_for_spawn).await;
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
            improve_mood,
            quit,
            get_cursor_position,
            get_accent_color,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Monitoring loop for auto-start
async fn super_monitoring_loop(
    app_handle: tauri::AppHandle,
    monitoring: std::sync::Arc<std::sync::Mutex<bool>>,
    config_lock: std::sync::Arc<tokio::sync::RwLock<lotaria::state::Config>>,
) {
    use tokio::time::{sleep, Duration};
    use lotaria::state::get_interval_seconds;

    tracing::info!("Monitoring loop started");

    loop {
        let is_active = match monitoring.lock() {
            Ok(g) => *g,
            Err(_) => {
                tracing::error!("Failed to lock monitoring mutex");
                break;
            }
        };
        
        if !is_active {
            tracing::info!("Monitoring stopped");
            break;
        }

        let interval_secs = {
            let config = config_lock.read().await;
            get_interval_seconds(
                &config.interval,
                config.gemini_free_tier,
                &config.tts_provider,
            )
        };

        tracing::info!("Waiting {} seconds until next roast...", interval_secs);
        sleep(Duration::from_secs(interval_secs)).await;

        let is_active = match monitoring.lock() {
            Ok(g) => *g,
            Err(_) => break,
        };
        
        if !is_active {
            break;
        }

        if let Err(e) = app_handle.emit("monitoring-tick", ()) {
            tracing::error!("Failed to emit monitoring tick: {}", e);
        }
    }

    tracing::info!("Monitoring loop ended");
}
