use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewWindow};

const MINI_WIDTH: f64 = 396.0;
const MINI_HEIGHT: f64 = 152.0;

const MIN_WIDTH: f64 = 300.0;
const MAX_WIDTH: f64 = 720.0;
const MIN_HEIGHT: f64 = 120.0;
const MAX_HEIGHT: f64 = 420.0;

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn size_of(width: Option<f64>, height: Option<f64>) -> (f64, f64) {
    (
        clamp(width.unwrap_or(MINI_WIDTH), MIN_WIDTH, MAX_WIDTH),
        clamp(height.unwrap_or(MINI_HEIGHT), MIN_HEIGHT, MAX_HEIGHT),
    )
}

fn window_of(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("main")
        .ok_or_else(|| "Окно не найдено".to_string())
}

fn force_size(win: &WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    let locked = !win.is_resizable().unwrap_or(false);
    if locked {
        let _ = win.set_resizable(true);
    }
    let size = LogicalSize::new(width, height);
    win.set_min_size(Some(size)).map_err(|e| e.to_string())?;
    win.set_max_size(Some(size)).map_err(|e| e.to_string())?;
    win.set_size(size).map_err(|e| e.to_string())?;
    if locked {
        let _ = win.set_resizable(false);
    }
    Ok(())
}

#[tauri::command]
pub fn enter_mini_player(
    app: AppHandle,
    width: Option<f64>,
    height: Option<f64>,
) -> Result<(), String> {
    let win = window_of(&app)?;
    let (width, height) = size_of(width, height);

    let _ = win.unmaximize();
    win.set_resizable(false).map_err(|e| e.to_string())?;
    force_size(&win, width, height)?;
    win.set_always_on_top(true).map_err(|e| e.to_string())?;

    if let Ok(Some(monitor)) = win.current_monitor() {
        let scale = monitor.scale_factor();
        let size = monitor.size().to_logical::<f64>(scale);
        let pos = monitor.position().to_logical::<f64>(scale);
        let x = pos.x + size.width - width - 24.0;
        let y = pos.y + size.height - height - 72.0;
        let _ = win.set_position(LogicalPosition::new(x, y));
    }

    let _ = win.show();
    let _ = win.set_focus();
    Ok(())
}

#[tauri::command]
pub fn resize_mini_player(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    let win = window_of(&app)?;
    let (width, height) = size_of(Some(width), Some(height));
    force_size(&win, width, height)
}

#[tauri::command]
pub fn exit_mini_player(app: AppHandle) -> Result<(), String> {
    let win = window_of(&app)?;

    win.set_always_on_top(false).map_err(|e| e.to_string())?;
    win.set_resizable(true).map_err(|e| e.to_string())?;
    win.set_max_size(None::<LogicalSize<f64>>)
        .map_err(|e| e.to_string())?;
    win.set_min_size(Some(LogicalSize::new(940.0, 600.0)))
        .map_err(|e| e.to_string())?;
    win.set_size(LogicalSize::new(1200.0, 800.0))
        .map_err(|e| e.to_string())?;
    let _ = win.center();
    let _ = win.set_focus();
    Ok(())
}
