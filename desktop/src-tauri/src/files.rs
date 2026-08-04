use std::fs;
use std::path::PathBuf;

fn default_dir() -> PathBuf {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let mut base = PathBuf::from(profile);
        base.push("Downloads");
        return base;
    }
    if let Ok(home) = std::env::var("HOME") {
        let mut base = PathBuf::from(home);
        base.push("Downloads");
        return base;
    }
    std::env::temp_dir()
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c == '\\' || c == '/' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<'
                || c == '>' || c == '|'
            {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim().to_string();
    if trimmed.is_empty() {
        "mashiro-export.txt".to_string()
    } else {
        trimmed
    }
}

#[tauri::command]
pub fn export_text_file(name: String, content: String, dir: Option<String>) -> Result<String, String> {
    let mut base = match dir {
        Some(value) if !value.trim().is_empty() => PathBuf::from(value.trim()),
        _ => default_dir(),
    };
    if !base.exists() {
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    }
    base.push(sanitize(&name));
    fs::write(&base, content).map_err(|e| e.to_string())?;
    Ok(base.to_string_lossy().to_string())
}

#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(path.trim()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    let target = url.trim();
    if !target.starts_with("http://") && !target.starts_with("https://") {
        return Err("unsupported url".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", target])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
