// Debug version with console window visible
// Build with: cargo build --release --features debug-console

use lotaria::{commands::*, state::StateManager};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, Emitter};

fn main() {
    println!("=== Lotaria Debug Build ===");
    
    // Set up panic hook
    std::panic::set_hook(Box::new(|info| {
        println!("PANIC: {:?}", info);
        let msg = format!("PANIC: {:?}\n", info);
        let _ = std::fs::write("lotaria_panic.log", msg);
    }));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            println!("Setup starting...");
            
            let state_manager = match StateManager::new() {
                Ok(sm) => {
                    println!("StateManager created");
                    sm
                }
                Err(e) => {
                    println!("StateManager error: {}", e);
                    return Err(e.to_string().into());
                }
            };
            
            let app_state = match AppState::new(state_manager) {
                Ok(state) => {
                    println!("AppState created");
                    state
                }
                Err(e) => {
                    println!("AppState error: {}", e);
                    return Err(e.to_string().into());
                }
            };

            let is_first_run = app_state.config.blocking_read().first_run;
            println!("First run: {}", is_first_run);

            let monitoring = app_state.monitoring.clone();
            let config_lock = app_state.config.clone();

            app.manage(app_state);

            println!("Creating window...");
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Lotaria")
                .inner_size(420.0, 400.0)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .resizable(true)
                .skip_taskbar(false)
                .visible(true)
                .build()?;

            println!("Window created!");

            if let Ok(Some(monitor)) = window.current_monitor() {
                let size = monitor.size();
                let window_size = window.inner_size()?;
                let x = size.width as i32 - window_size.width as i32 - 20;
                let y = size.height as i32 - window_size.height as i32 - 20;
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
            }

            println!("Setup complete!");
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
            quit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn super_monitoring_loop(
    app_handle: tauri::AppHandle,
    monitoring: std::sync::Arc<std::sync::Mutex<bool>>,
    config_lock: std::sync::Arc<tokio::sync::RwLock<lotaria::state::Config>>,
) {
    use tokio::time::{sleep, Duration};
    use lotaria::state::get_interval_seconds;

    loop {
        let is_active = monitoring.lock().map(|g| *g).unwrap_or(false);
        if !is_active {
            break;
        }

        let config = config_lock.read().await;
        let interval_secs = get_interval_seconds(
            &config.interval,
            config.gemini_free_tier,
            &config.tts_provider,
        );
        drop(config);

        sleep(Duration::from_secs(interval_secs)).await;

        let is_active = monitoring.lock().map(|g| *g).unwrap_or(false);
        if !is_active {
            break;
        }

        let _ = app_handle.emit("monitoring-tick", ());
    }
}
