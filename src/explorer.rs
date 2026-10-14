use std::path::Path;

/// Opens the given path directly in the system's native file explorer
/// (Windows File Explorer, macOS Finder, or Linux file manager).
pub fn open_in_file_explorer(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Cannot open path: '{}' does not exist.", path.display()));
    }

    let path_str = path.to_string_lossy();
    let target = if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        stripped
    } else {
        &path_str
    };

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(target)
            .spawn()
            .map_err(|e| format!("Failed to open Windows File Explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open macOS Finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg("--")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
    }

    Ok(())
}
