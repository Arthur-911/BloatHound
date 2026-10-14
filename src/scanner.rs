use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{get_default_patterns, PROTECTED_SYSTEM_DIRS};
use crate::inspector::inspect_artifact;
use crate::models::ArtifactInfo;

pub fn validate_scan_path(target: &Path) -> Result<PathBuf, String> {
    let canonical = target
        .canonicalize()
        .map_err(|e| format!("Target path cannot be accessed: {}", e))?;

    if !canonical.is_dir() {
        return Err(format!("Target path is not a directory: {}", canonical.display()));
    }

    // Safety: check root path
    if canonical.parent().is_none() {
        return Err(format!(
            "Scanning root drive ({}) is blocked for safety. Please choose a specific folder.",
            canonical.display()
        ));
    }

    // Safety: check protected system folders
    let dir_name_lower = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if PROTECTED_SYSTEM_DIRS.contains(&dir_name_lower.as_str()) {
        return Err(format!(
            "Scanning protected system directory ({}) is blocked for safety.",
            canonical.display()
        ));
    }

    let mut s = canonical.to_string_lossy().to_string();
    if s.starts_with(r"\\?\") {
        s = s[4..].to_string();
    }
    let clean = PathBuf::from(s);

    Ok(clean)
}

pub fn scan_directory(root: &Path, max_depth: usize) -> Result<Vec<ArtifactInfo>, String> {
    let valid_root = validate_scan_path(root)?;
    let patterns = get_default_patterns();
    let mut artifacts = Vec::new();

    let mut it = WalkDir::new(&valid_root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter();

    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(_)) => continue,
            Some(Ok(entry)) => entry,
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy();
        let name_lower = name.to_lowercase();

        // Skip git and protected directories
        if name == ".git" || PROTECTED_SYSTEM_DIRS.contains(&name_lower.as_str()) {
            it.skip_current_dir();
            continue;
        }

        // Check if directory matches any known disposable artifact pattern
        if let Some(pattern) = patterns.get(name.as_ref()) {
            let path = entry.path();
            let info = inspect_artifact(path, pattern);
            artifacts.push(info);

            // Crucial performance optimization: skip descending into this artifact directory!
            it.skip_current_dir();
        }
    }

    Ok(artifacts)
}
