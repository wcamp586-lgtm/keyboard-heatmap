use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

use crate::{app_state::AppState, models::PrivacyStatus};

const TRAY_ID: &str = "keyboard-heatmap-tray";
const SHOW_ID: &str = "show";
const TOGGLE_COLLECTION_ID: &str = "toggle_collection";
const QUIT_ID: &str = "quit";

/// Installs the deliberately small V1 tray menu: show, pause/resume, quit.
pub fn setup<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, SHOW_ID, "显示键盘热力", true, None::<&str>)?;
    let status = app.state::<AppState>().privacy_status();
    let (toggle_text, tooltip) = status_presentation(&status);
    let toggle = MenuItem::with_id(app, TOGGLE_COLLECTION_ID, toggle_text, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &toggle, &quit])?;
    let toggle_for_menu = toggle.clone();
    let toggle_for_icon = toggle.clone();

    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon(heatmap_tray_icon())
        .tooltip(tooltip)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            SHOW_ID => {
                show_main_window(app);
                update_tray_status(app, &toggle_for_menu);
            }
            TOGGLE_COLLECTION_ID => {
                let status = app.state::<AppState>().privacy_status();
                // If the receiver stopped while the collection gate is still
                // on, this item becomes a recovery attempt rather than a
                // misleading "pause" action.
                let should_enable = !status.collection_enabled || !status.raw_input_active;
                let _ = app
                    .state::<AppState>()
                    .set_collection_enabled(should_enable);
                update_tray_status(app, &toggle_for_menu);
            }
            QUIT_ID => {
                app.state::<AppState>().prepare_for_explicit_exit();
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| {
            let should_show = matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            );

            if should_show {
                let app = tray.app_handle();
                show_main_window(app);
                update_tray_status(app, &toggle_for_icon);
            }
        })
        .build(app)?;

    Ok(())
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.show().is_ok() {
            let _ = window.set_focus();
            app.state::<AppState>().note_window_shown();
        }
    } else {
        recover_main_window(app);
    }
}

/// Rebuilds the configured main window if it was unexpectedly destroyed.
///
/// Tauri documents a WebView2 deadlock hazard when a Windows webview is
/// created synchronously inside an event handler. This function therefore
/// schedules the exact configured main window from a short-lived worker
/// thread, while an AppState gate prevents duplicate recoveries.
pub fn recover_main_window<R: Runtime>(app: &AppHandle<R>) {
    if !app.state::<AppState>().begin_main_window_recovery() {
        return;
    }

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some("键盘热力 — 正在恢复主窗口"));
    }

    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned();
    let app = app.clone();
    std::thread::spawn(move || {
        // The native Destroyed callback happens just before Tauri removes the
        // old label from its window registry. Give that removal a moment, then
        // retry a few times in case the callback and worker race on Windows.
        let mut succeeded = false;
        if let Some(config) = config {
            std::thread::sleep(std::time::Duration::from_millis(50));
            for attempt in 0..5 {
                match tauri::WebviewWindowBuilder::from_config(&app, &config)
                    .and_then(|builder| builder.build())
                {
                    Ok(window) if window.show().is_ok() => {
                        let _ = window.set_focus();
                        app.state::<AppState>().note_window_shown();
                        succeeded = true;
                        break;
                    }
                    Ok(_) | Err(_) if attempt < 4 => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Ok(_) | Err(_) => break,
                }
            }
        }

        app.state::<AppState>()
            .finish_main_window_recovery(succeeded);
        if succeeded {
            update_tray_tooltip(&app);
        } else if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = tray.set_tooltip(Some("键盘热力 — 主窗口恢复失败，可再次点击托盘重试"));
        }
    });
}

fn update_tray_status<R: Runtime>(app: &AppHandle<R>, toggle: &MenuItem<R>) {
    let status = app.state::<AppState>().privacy_status();
    let (toggle_text, _) = status_presentation(&status);
    let _ = toggle.set_text(toggle_text);
    update_tray_tooltip(app);
}

fn update_tray_tooltip<R: Runtime>(app: &AppHandle<R>) {
    let status = app.state::<AppState>().privacy_status();
    let (_, tooltip) = status_presentation(&status);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

fn status_presentation(status: &PrivacyStatus) -> (&'static str, &'static str) {
    match (status.collection_enabled, status.raw_input_active) {
        (true, true) => ("暂停统计", "键盘热力 — 正在统计（本地聚合）"),
        (false, _) => ("继续统计", "键盘热力 — 已暂停统计"),
        (true, false) => (
            "尝试恢复统计（采集异常）",
            "键盘热力 — 采集异常，请打开主窗口查看",
        ),
    }
}

/// A tiny runtime-generated icon keeps the tray independent of any binary icon
/// asset while still giving the Windows notification area a visible indicator.
fn heatmap_tray_icon() -> Image<'static> {
    const SIZE: usize = 16;
    let mut rgba = vec![0_u8; SIZE * SIZE * 4];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let index = (y * SIZE + x) * 4;
            let hot = (x + y * 2) % 5;
            let (red, green, blue) = match hot {
                0 => (242, 120, 161),
                1 => (222, 91, 140),
                _ => (45, 47, 55),
            };
            let border = x == 0 || y == 0 || x == SIZE - 1 || y == SIZE - 1;
            rgba[index..index + 4].copy_from_slice(&[
                if border { 26 } else { red },
                if border { 28 } else { green },
                if border { 33 } else { blue },
                255,
            ]);
        }
    }

    Image::new_owned(rgba, SIZE as u32, SIZE as u32)
}

#[cfg(test)]
mod tests {
    use super::status_presentation;
    use crate::models::PrivacyStatus;

    #[test]
    fn tray_copy_matches_collection_health() {
        assert_eq!(
            status_presentation(&PrivacyStatus::new(true, true)),
            ("暂停统计", "键盘热力 — 正在统计（本地聚合）")
        );
        assert_eq!(
            status_presentation(&PrivacyStatus::new(false, false)),
            ("继续统计", "键盘热力 — 已暂停统计")
        );
        assert_eq!(
            status_presentation(&PrivacyStatus::new(true, false)),
            (
                "尝试恢复统计（采集异常）",
                "键盘热力 — 采集异常，请打开主窗口查看"
            )
        );
    }
}
