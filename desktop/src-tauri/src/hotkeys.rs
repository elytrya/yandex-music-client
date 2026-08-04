use serde::Deserialize;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(Debug, Deserialize)]
pub struct GlobalBinding {
    pub action: String,
    pub accelerator: String,
}

#[tauri::command]
pub fn set_global_hotkeys<R: Runtime>(
    app: AppHandle<R>,
    bindings: Vec<GlobalBinding>,
) -> Result<Vec<String>, String> {
    let manager = app.global_shortcut();
    let _ = manager.unregister_all();

    let mut failed: Vec<String> = Vec::new();

    for binding in bindings {
        let accelerator = binding.accelerator.trim().to_string();
        if accelerator.is_empty() {
            continue;
        }

        let event_name = format!("hotkey://{}", binding.action);
        let result = manager.on_shortcut(accelerator.as_str(), move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = app.emit(event_name.as_str(), ());
            }
        });

        if result.is_err() {
            failed.push(accelerator);
        }
    }

    Ok(failed)
}

#[tauri::command]
pub fn clear_global_hotkeys<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|error| error.to_string())
}
