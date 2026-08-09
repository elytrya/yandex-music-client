use std::fs::{create_dir_all, metadata, remove_file, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

const LIMIT: u64 = 1024 * 1024;

fn stamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let day = now.as_secs() % 86_400;

    format!(
        "{:02}:{:02}:{:02}.{:03}",
        day / 3600,
        (day % 3600) / 60,
        day % 60,
        now.subsec_millis()
    )
}

pub fn path(app: &AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_log_dir().ok()?;
    create_dir_all(&dir).ok()?;
    Some(dir.join("together.log"))
}

fn append(app: &AppHandle, line: &str) {
    let Some(file_path) = path(app) else {
        return;
    };

    if let Ok(info) = metadata(&file_path) {
        if info.len() > LIMIT {
            let _ = remove_file(&file_path);
        }
    }

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
    {
        let _ = writeln!(file, "{line}");
    }
}

pub fn write(app: &AppHandle, scope: &str, text: &str) {
    let time = stamp();
    let line = format!("{time} [together:{scope}] {text}");

    eprintln!("{line}");
    append(app, &line);

    let _ = app.emit(
        "together://log",
        json!({ "time": time, "scope": scope, "text": text }),
    );
}
