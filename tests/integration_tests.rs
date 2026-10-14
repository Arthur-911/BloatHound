use dev_cleaner::cleaner::purge_artifact;
use dev_cleaner::config::get_default_patterns;
use dev_cleaner::inspector::{get_path_timestamps, inspect_artifact, list_files_with_timestamps};
use dev_cleaner::models::{format_bytes, format_time_relative, ArtifactInfo};
use dev_cleaner::scanner::scan_directory;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_models_format_bytes() {
    assert_eq!(format_bytes(500), "500 B");
    assert_eq!(format_bytes(2048), "2 KB");
    assert_eq!(format_bytes(10 * 1024 * 1024), "10.0 MB");
    assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2.0 GB");
}

#[test]
fn test_models_format_relative_time() {
    let now = chrono::Local::now();
    let formatted = format_time_relative(Some(now));
    assert!(formatted.contains("just now") || formatted.contains("0s ago"));
}

#[test]
fn test_inspector_timestamps_and_size() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello Rust World").unwrap();
    }

    let (created, accessed, modified) = get_path_timestamps(&file_path);
    assert!(created.is_some());
    assert!(accessed.is_some());
    assert!(modified.is_some());

    let files = list_files_with_timestamps(dir.path(), 10);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].size_bytes, 16);
    assert!(files[0].created_at.is_some());
}

#[test]
fn test_scanner_detection_and_pruning() {
    let root = tempdir().unwrap();
    let project = root.path().join("my_app");
    fs::create_dir(&project).unwrap();

    // Create a mock node_modules
    let node_modules = project.join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    let dummy_pkg = node_modules.join("pkg.js");
    fs::write(dummy_pkg, "module.exports = {};").unwrap();

    // Create a mock .venv
    let venv = project.join(".venv");
    fs::create_dir(&venv).unwrap();
    fs::write(venv.join("cfg.txt"), "home = /usr").unwrap();

    // Create a .git directory (should be ignored)
    let git = project.join(".git");
    fs::create_dir(&git).unwrap();
    fs::write(git.join("HEAD"), "ref: refs/heads/main").unwrap();

    let artifacts = scan_directory(root.path(), 10).unwrap();
    let patterns: Vec<String> = artifacts.iter().map(|a| a.pattern.clone()).collect();

    assert!(patterns.contains(&"node_modules".to_string()));
    assert!(patterns.contains(&".venv".to_string()));
    assert!(!patterns.contains(&".git".to_string()));
    assert_eq!(artifacts.len(), 2);
}

#[test]
fn test_cleaner_dry_run_and_actual_purge() {
    let root = tempdir().unwrap();
    let nm = root.path().join("node_modules");
    fs::create_dir(&nm).unwrap();
    fs::write(nm.join("index.js"), "console.log('hi');").unwrap();

    let pattern = get_default_patterns().get("node_modules").unwrap().clone();
    let info = inspect_artifact(&nm, &pattern);

    // Dry run
    purge_artifact(&info, true).unwrap();
    assert!(nm.exists(), "Directory should still exist after dry run");

    // Actual purge
    purge_artifact(&info, false).unwrap();
    assert!(!nm.exists(), "Directory should be removed after actual purge");
}

#[test]
fn test_cleaner_safety_block_on_arbitrary_folder() {
    let root = tempdir().unwrap();
    let dangerous_dir = root.path().join("important_user_code");
    fs::create_dir(&dangerous_dir).unwrap();

    let info = ArtifactInfo {
        path: dangerous_dir.clone(),
        project_name: "test".to_string(),
        category: "Code".to_string(),
        pattern: "important_user_code".to_string(),
        size_bytes: 100,
        file_count: 1,
        created_at: None,
        accessed_at: None,
        modified_at: None,
    };

    let result = purge_artifact(&info, false);
    assert!(result.is_err());
    assert!(dangerous_dir.exists(), "User code must remain untouched");
}
