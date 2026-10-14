use chrono::{DateTime, Local};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::ArtifactPattern;
use crate::models::{ArtifactInfo, FileDetail};

pub type PathTimestamps = (
    Option<DateTime<Local>>,
    Option<DateTime<Local>>,
    Option<DateTime<Local>>,
);

pub fn get_path_timestamps(path: &Path) -> PathTimestamps {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return (None, None, None),
    };

    let created: Option<DateTime<Local>> = metadata.created().ok().map(DateTime::<Local>::from);
    let accessed: Option<DateTime<Local>> = metadata.accessed().ok().map(DateTime::<Local>::from);
    let modified: Option<DateTime<Local>> = metadata.modified().ok().map(DateTime::<Local>::from);

    (created, accessed, modified)
}

pub fn inspect_artifact(artifact_path: &Path, pattern: &ArtifactPattern) -> ArtifactInfo {
    let mut total_size: u64 = 0;
    let mut file_count: usize = 0;

    let (dir_created, dir_accessed, dir_modified) = get_path_timestamps(artifact_path);

    let mut max_accessed_st: Option<std::time::SystemTime> = None;
    let mut max_modified_st: Option<std::time::SystemTime> = None;

    for entry in WalkDir::new(artifact_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path_is_symlink() {
            continue;
        }
        if entry.file_type().is_file() {
            file_count += 1;
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();

                // High-performance optimization: sample timestamps up to depth 3
                // comparing raw SystemTime rather than calling DateTime conversions on every file
                if entry.depth() <= 3 {
                    if let Ok(acc) = metadata.accessed() {
                        if max_accessed_st.is_none_or(|cur| acc > cur) {
                            max_accessed_st = Some(acc);
                        }
                    }

                    if let Ok(mod_time) = metadata.modified() {
                        if max_modified_st.is_none_or(|cur| mod_time > cur) {
                            max_modified_st = Some(mod_time);
                        }
                    }
                }
            }
        }
    }

    let max_accessed = max_accessed_st
        .map(DateTime::<Local>::from)
        .or(dir_accessed);
    let max_modified = max_modified_st
        .map(DateTime::<Local>::from)
        .or(dir_modified);

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
        if entry.path_is_symlink() {
            continue;
        }
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
    files.sort_by_key(|a| std::cmp::Reverse(a.size_bytes));

    if files.len() > max_files {
        files.truncate(max_files);
    }

    files
}
