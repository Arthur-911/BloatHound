use std::fs;
use std::path::Path;

use crate::config::get_default_patterns;
use crate::models::{format_bytes, ArtifactInfo};

#[derive(Debug, Default)]
pub struct CleanResult {
    pub success_count: usize,
    pub error_count: usize,
    pub bytes_freed: u64,
    pub dry_run: bool,
    pub errors: Vec<String>,
}

impl CleanResult {
    pub fn human_freed_size(&self) -> String {
        format_bytes(self.bytes_freed)
    }
}

pub fn remove_dir_all_resilient(path: &Path) -> std::io::Result<()> {
    if let Err(e) = fs::remove_dir_all(path) {
        // If removal failed, attempt to clear read-only attributes and retry
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                let mut perms = metadata.permissions();

                #[cfg(windows)]
                {
                    if perms.readonly() {
                        #[allow(clippy::permissions_set_readonly_false)]
                        perms.set_readonly(false);
                        let _ = fs::set_permissions(entry.path(), perms);
                    }
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = perms.mode();
                    if mode & 0o200 == 0 {
                        perms.set_mode(mode | 0o200);
                        let _ = fs::set_permissions(entry.path(), perms);
                    }
                }
            }
        }
        // Retry removal
        fs::remove_dir_all(path).map_err(|_| e)?;
    }
    Ok(())
}

pub fn purge_artifact(info: &ArtifactInfo, dry_run: bool) -> Result<(), String> {
    let path = &info.path;

    if !path.exists() {
        return Ok(());
    }

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    // Safety guard: ensure the folder name is strictly in the known patterns list
    let folder_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid directory name".to_string())?;

    let patterns = get_default_patterns();
    if !patterns.contains_key(folder_name) {
        return Err(format!(
            "Safety block: Directory '{}' is not in the safe disposable list.",
            folder_name
        ));
    }

    if dry_run {
        return Ok(());
    }

    remove_dir_all_resilient(path).map_err(|e| format!("Failed to delete {}: {}", path.display(), e))
}

pub fn purge_artifacts(artifacts: &[ArtifactInfo], dry_run: bool) -> CleanResult {
    let mut result = CleanResult {
        dry_run,
        ..Default::default()
    };

    for item in artifacts {
        match purge_artifact(item, dry_run) {
            Ok(_) => {
                result.success_count += 1;
                result.bytes_freed += item.size_bytes;
            }
            Err(e) => {
                result.error_count += 1;
                result.errors.push(e);
            }
        }
    }

    result
}
