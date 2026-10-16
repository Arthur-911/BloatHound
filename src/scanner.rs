use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{find_pattern, PROTECTED_SYSTEM_DIRS};
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
    if canonical.parent().is_none()
        || canonical
            .components()
            .filter(|c| matches!(c, std::path::Component::Normal(_)))
            .count()
            == 0
    {
        return Err(format!(
            "Scanning root drive ({}) is blocked for safety. Please choose a specific folder.",
            canonical.display()
        ));
    }

    // Safety: check protected system folders across all ancestor components
    for comp in canonical.components() {
        if let std::path::Component::Normal(os_str) = comp {
            let name_lower = os_str.to_string_lossy().to_lowercase();
            if PROTECTED_SYSTEM_DIRS.contains(&name_lower.as_str()) {
                return Err(format!(
                    "Scanning inside protected system directory ({}) is blocked for safety.",
                    canonical.display()
                ));
            }
        }
    }

    let mut s = canonical.to_string_lossy().to_string();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        s = stripped.to_string();
    }
    let clean = PathBuf::from(s);

    Ok(clean)
}

pub fn scan_directory(root: &Path, max_depth: usize) -> Result<Vec<ArtifactInfo>, String> {
    let valid_root = validate_scan_path(root)?;
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
        if let Some(pattern) = find_pattern(name.as_ref()) {
            let path = entry.path();
            let info = inspect_artifact(path, pattern);
            artifacts.push(info);

            // Crucial performance optimization: skip descending into this artifact directory!
            it.skip_current_dir();
        }
    }

    // Sort by largest artifact size first so top results are most impactful
    artifacts.sort_by_key(|a| std::cmp::Reverse(a.size_bytes));

    Ok(artifacts)
}
