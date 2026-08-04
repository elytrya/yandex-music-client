use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager};

const MINI_WIDTH: f64 = 396.0;
const MINI_HEIGHT: f64 = 152.0;

#[tauri::command]
pub fn enter_mini_player(app: AppHandle) -> Result<(), String> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| "Окно не найдено".to_string())?;

    let _ = win.unmaximize();
    win.set_min_size(None::<LogicalSize<f64>>)
        .map_err(|e| e.to_string())?;
    win.set_resizable(false).map_err(|e| e.to_string())?;
    win.set_size(LogicalSize::new(MINI_WIDTH, MINI_HEIGHT))
        .map_err(|e| e.to_string())?;
    win.set_always_on_top(true).map_err(|e| e.to_string())?;

    if let Ok(Some(monitor)) = win.current_monitor() {
        let scale = monitor.scale_factor();
        let size = monitor.size().to_logical::<f64>(scale);
        let pos = monitor.position().to_logical::<f64>(scale);
        let x = pos.x + size.width - MINI_WIDTH - 24.0;
        let y = pos.y + size.height - MINI_HEIGHT - 72.0;
        let _ = win.set_position(LogicalPosition::new(x, y));
    }

    let _ = win.show();
    let _ = win.set_focus();
    Ok(())
}

#[tauri::command]
pub fn exit_mini_player(app: AppHandle) -> Result<(), String> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| "Окно не найдено".to_string())?;

    win.set_always_on_top(false).map_err(|e| e.to_string())?;
    win.set_resizable(true).map_err(|e| e.to_string())?;
    win.set_min_size(Some(LogicalSize::new(940.0, 600.0)))
        .map_err(|e| e.to_string())?;
    win.set_size(LogicalSize::new(1200.0, 800.0))
        .map_err(|e| e.to_string())?;
    let _ = win.center();
    let _ = win.set_focus();
    Ok(())
}
