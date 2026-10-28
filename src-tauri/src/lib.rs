#[cfg(target_os = "macos")]
pub mod agent_controller;
pub mod app_state;
pub mod collector;
pub mod commands;
pub mod date_range;
pub mod keymap;
pub mod lifecycle;
pub mod models;
pub mod raw_input;
pub mod shortcut_hook;
pub mod shortcuts;
pub mod storage;
pub mod tray;

use tauri::Manager;

use app_state::AppState;

/// Starts the Windows desktop application.
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(state) = app.try_state::<AppState>() {
                state.note_second_instance_redirected();
            }
            tray::show_main_window(app);
        }))
        .on_window_event(|window, event| {
            if window.label() == "main" {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        // Closing the dashboard keeps the local-only collector
                        // alive; the explicit tray Exit action is the only path
                        // that quits.
                        api.prevent_close();
                        if window.hide().is_ok() {
                            window.app_handle().state::<AppState>().note_window_hidden();
                        }
                    }
                    tauri::WindowEvent::Destroyed => {
                        // A native/WebView failure can destroy the main UI
                        // through a path that bypasses CloseRequested. Keep
                        // the tray process recoverable instead of leaving a
                        // hidden collector with no dashboard to reopen.
                        tray::recover_main_window(window.app_handle());
                    }
                    _ => {}
                }
            }
        })
        .setup(|app| {
            let state = AppState::initialize(app.handle())?;
            app.manage(state);
            tray::setup(app.handle())?;
            if let Some(window) = app.get_webview_window("main") {
                tray::set_version_title(&window)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_snapshot,
            commands::get_dashboard,
            commands::get_history,
            commands::get_day_detail,
            commands::get_live_counts,
            commands::set_collection_enabled,
            commands::get_privacy_status,
        ])
        .build(tauri::generate_context!())
        .expect("Could not build the Keyboard Heatmap application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
            let state = app_handle.state::<AppState>();
            if code.is_none() && !state.explicit_exit_requested() {
                // Tauri documents `code: None` as a user-interaction exit
                // request. The app has a persistent tray receiver, so keep
                // the collector alive unless the user chose the explicit tray
                // Exit action. Programmatic exits keep their `Some(code)` and
                // are intentionally not intercepted.
                //
                // On Windows, the Wry runtime does not translate
                // WM_QUERYENDSESSION into this event; normal system shutdown
                // therefore remains under Windows' shutdown handling.
                state.note_unintended_exit_prevented();
                api.prevent_exit();
            } else {
                // Flush only aggregates that were already accepted. No raw
                // input, device state, or text is written during shutdown.
                state.collector.shutdown();
            }
        }
    });
}
