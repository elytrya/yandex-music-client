use std::sync::atomic::{AtomicBool, Ordering};

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};

static CLOSE_TO_TRAY: AtomicBool = AtomicBool::new(true);

pub const TRAY_ID: &str = "mashiro-tray";

pub fn close_to_tray() -> bool {
    CLOSE_TO_TRAY.load(Ordering::Relaxed)
}

fn reveal<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = app.emit("app://shown", ());
    }
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        let _ = app.emit("app://hidden", ());
    }
}

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "tray_show", "Открыть Mashiro", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "tray_toggle", "Пауза / Продолжить", true, None::<&str>)?;
    let prev = MenuItem::with_id(app, "tray_prev", "Предыдущий трек", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "tray_next", "Следующий трек", true, None::<&str>)?;
    let like = MenuItem::with_id(app, "tray_like", "Нравится", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray_quit", "Выйти", true, None::<&str>)?;
    let sep_one = PredefinedMenuItem::separator(app)?;
    let sep_two = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &sep_one,
            &toggle,
            &prev,
            &next,
            &like,
            &sep_two,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Mashiro")
        .menu(&menu);

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            "tray_show" => reveal(app),
            "tray_toggle" => {
                let _ = app.emit("tray://toggle", ());
            }
            "tray_prev" => {
                let _ = app.emit("tray://prev", ());
            }
            "tray_next" => {
                let _ = app.emit("tray://next", ());
            }
            "tray_like" => {
                let _ = app.emit("tray://like", ());
            }
            "tray_quit" => {
                let _ = app.emit("tray://quit", ());
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                reveal(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

#[tauri::command]
pub fn set_close_to_tray(enabled: bool) {
    CLOSE_TO_TRAY.store(enabled, Ordering::Relaxed);
}

#[tauri::command]
pub fn set_tray_tooltip(app: AppHandle, text: String) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let value = if text.trim().is_empty() {
            "Mashiro".to_string()
        } else {
            text
        };
        tray.set_tooltip(Some(value)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_to_tray(app: AppHandle) {
    hide(&app);
}

#[tauri::command]
pub fn show_from_tray(app: AppHandle) {
    reveal(&app);
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
