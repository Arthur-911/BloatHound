use std::fs;
use walkdir::WalkDir;

use crate::models::ArtifactInfo;

/// Analysis of whether an artifact folder is truly needed or if deleting it has zero negative impact.
#[derive(Debug, Clone)]
pub struct IntegrityAnalysis {
    /// Is this folder connected to a valid project manifest (package.json, Cargo.toml, requirements.txt)?
    pub has_manifest: bool,
    pub manifest_file: Option<String>,

    /// Does this folder contain custom/unique user files that would cause permanent data loss if deleted?
    pub has_custom_user_files: bool,
    pub custom_files_found: Vec<String>,

    /// Is this folder or its contents directly imported or referenced by the project's source code?
    pub is_referenced_in_source: bool,
    pub source_references_found: Vec<String>,

    /// Final Verdict: Is it needed or does it NOT matter if deleted?
    pub is_needed: bool,
    pub verdict_summary: String,
}

const PROJECT_MANIFESTS: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "requirements.txt",
    "Pipfile",
    "pom.xml",
    "build.gradle",
    "go.mod",
    "CMakeLists.txt",
];

const SOURCE_EXTENSIONS: &[&str] = &[
    "js", "ts", "jsx", "tsx", "py", "rs", "go", "java", "c", "cpp", "h", "cs", "html",
];

const SENSITIVE_USER_EXTENSIONS: &[&str] = &[
    "env", "sql", "sqlite", "db", "pem", "key", "cert", "secret", "draft", "kdbx", "ovpn",
];

pub fn analyze_folder_integrity(artifact: &ArtifactInfo) -> IntegrityAnalysis {
    let artifact_dir = &artifact.path;
    let parent_dir = artifact_dir.parent().unwrap_or(artifact_dir);

    // 1. MANIFEST CONNECTION CHECK
    // Check if there is an active project manifest in the parent or grandparent folder
    let mut manifest_found = None;
    for manifest in PROJECT_MANIFESTS {
        let parent_manifest = parent_dir.join(manifest);
        if parent_manifest.exists() {
            manifest_found = Some(manifest.to_string());
            break;
        }
        if let Some(grandparent) = parent_dir.parent() {
            let gp_manifest = grandparent.join(manifest);
            if gp_manifest.exists() {
                manifest_found = Some(manifest.to_string());
                break;
            }
        }
    }

    let has_manifest = manifest_found.is_some();

    // 2. CUSTOM UN-GENERATED FILE SCAN
    // Check if the user accidentally stored unique personal files inside this folder (.env, .sql, .pem, etc.)
    let mut custom_files = Vec::new();
    for entry in WalkDir::new(artifact_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path_is_symlink() {
            continue;
        }
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            let ext = entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Check for explicit user environment configs, ssh keys, or sensitive non-build files
            let is_user_env = file_name == ".env"
                || file_name.starts_with(".env.")
                || file_name == ".envrc"
                || file_name == "credentials.json"
                || file_name == "id_rsa"
                || file_name == "id_ed25519";
            let is_user_sensitive = ext != "pdb" && SENSITIVE_USER_EXTENSIONS.contains(&ext.as_str());

            if is_user_env || is_user_sensitive {
                custom_files.push(entry.path().display().to_string());
            }

            if custom_files.len() >= 5 {
                break;
            }
        }
    }
    let has_custom_user_files = !custom_files.is_empty();

    // 3. SOURCE CODE REFERENCE / IMPORT CHECK
    // Scan true source files (*.js, *.py, *.rs, etc.) to see if user code hard-imports from this folder
    let folder_name = artifact.pattern.as_str();
    let mut source_references = Vec::new();

    if parent_dir.exists() {
        for entry in WalkDir::new(parent_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.starts_with(artifact_dir) {
                continue;
            }

            if path.is_file() {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if SOURCE_EXTENSIONS.contains(&ext) && !path.ends_with("Cargo.lock") {
                    // Safety guard: cap file size to 2 MB to avoid memory exhaustion on large bundles
                    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    if file_size > 2 * 1024 * 1024 {
                        continue;
                    }

                    if let Ok(content) = fs::read_to_string(path) {
                        // Look for explicit runtime import or require statements
                        let import_pattern_a = format!("require(\"./{}\"", folder_name);
                        let import_pattern_b = format!("from \"./{}\"", folder_name);
                        let import_pattern_c = format!("from './{}'", folder_name);
                        let import_pattern_d = format!("import \"./{}\"", folder_name);

                        if content.contains(&import_pattern_a)
                            || content.contains(&import_pattern_b)
                            || content.contains(&import_pattern_c)
                            || content.contains(&import_pattern_d)
                        {
                            source_references.push(path.display().to_string());
                            if source_references.len() >= 3 {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    let is_referenced_in_source = !source_references.is_empty();

    // 4. FINAL VERDICT COMPUTATION: Does it matter if deleted?
    let is_needed: bool;
    let verdict_summary: String;

    if has_custom_user_files {
        is_needed = true;
        verdict_summary = format!(
            "MATTERS! Contains {} custom user/config files (e.g. .env or databases). DO NOT DELETE.",
            custom_files.len()
        );
    } else if is_referenced_in_source && !has_manifest {
        is_needed = true;
        verdict_summary = format!(
            "MATTERS! Directly referenced by source code in '{}' without a build manifest to regenerate it.",
            source_references.first().cloned().unwrap_or_default()
        );
    } else if !has_manifest {
        // Orphaned cache folder with no project manifest attached
        is_needed = false;
        verdict_summary = "DOES NOT MATTER. Orphaned cache folder (no project manifest found). 100% safe to delete.".to_string();
    } else {
        // Connected to manifest, no custom files, standard disposable build folder
        is_needed = false;
        verdict_summary = format!(
            "DOES NOT MATTER. Recreatable cache connected to '{}'. Deleting will not harm project.",
            manifest_found.as_deref().unwrap_or("project")
        );
    }

    IntegrityAnalysis {
        has_manifest,
        manifest_file: manifest_found,
        has_custom_user_files,
        custom_files_found: custom_files,
        is_referenced_in_source,
        source_references_found: source_references,
        is_needed,
        verdict_summary,
    }
}
