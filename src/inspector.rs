use chrono::{DateTime, Local};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::ArtifactPattern;
use crate::models::{ArtifactInfo, FileDetail};

pub fn get_path_timestamps(
    path: &Path,
) -> (
    Option<DateTime<Local>>,
    Option<DateTime<Local>>,
    Option<DateTime<Local>>,
) {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return (None, None, None),
    };

    let created: Option<DateTime<Local>> = metadata
        .created()
        .ok()
        .map(|t| DateTime::<Local>::from(t));

    let accessed: Option<DateTime<Local>> = metadata
        .accessed()
        .ok()
        .map(|t| DateTime::<Local>::from(t));

    let modified: Option<DateTime<Local>> = metadata
        .modified()
        .ok()
        .map(|t| DateTime::<Local>::from(t));

    (created, accessed, modified)
}

pub fn inspect_artifact(artifact_path: &Path, pattern: &ArtifactPattern) -> ArtifactInfo {
    let mut total_size: u64 = 0;
    let mut file_count: usize = 0;

    let (dir_created, dir_accessed, dir_modified) = get_path_timestamps(artifact_path);

    let mut max_accessed = dir_accessed;
    let mut max_modified = dir_modified;

    for entry in WalkDir::new(artifact_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            file_count += 1;
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();

                if let Ok(acc) = metadata.accessed() {
                    let acc_dt: DateTime<Local> = acc.into();
                    if max_accessed.map_or(true, |cur| acc_dt > cur) {
                        max_accessed = Some(acc_dt);
                    }
                }

                if let Ok(mod_time) = metadata.modified() {
                    let mod_dt: DateTime<Local> = mod_time.into();
                    if max_modified.map_or(true, |cur| mod_dt > cur) {
                        max_modified = Some(mod_dt);
                    }
                }
            }
        }
    }

    let project_name = artifact_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    ArtifactInfo {
        path: artifact_path.to_path_buf(),
        project_name,
        category: pattern.category.to_string(),
        pattern: pattern.name.to_string(),
        size_bytes: total_size,
        file_count,
        created_at: dir_created,
        accessed_at: max_accessed,
        modified_at: max_modified,
        recreate_cmd: pattern.recreate_cmd.to_string(),
    }
}

pub fn list_files_with_timestamps(dir_path: &Path, max_files: usize) -> Vec<FileDetail> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path().to_path_buf();
            let relative_path = path
                .strip_prefix(dir_path)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| path.display().to_string());

            let (created, accessed, modified) = get_path_timestamps(&path);
            let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);

            files.push(FileDetail {
                path,
                relative_path,
                size_bytes,
                created_at: created,
                accessed_at: accessed,
                modified_at: modified,
            });
        }
    }

    // Sort by largest file size first
    files.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    if files.len() > max_files {
        files.truncate(max_files);
    }

    files
}
