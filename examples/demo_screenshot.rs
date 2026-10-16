use bloathound::models::ArtifactInfo;
use bloathound::ui;
use chrono::{Duration, Local};
use colored::*;
use std::path::PathBuf;

fn main() {
    colored::control::set_override(true);

    let now = Local::now();

    let items = vec![
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\ecommerce-platform\target"),
            project_name: "ecommerce-platform".to_string(),
            category: "Rust".to_string(),
            pattern: "target".to_string(),
            size_bytes: 3_435_973_836, // 3.2 GB
            file_count: 4210,
            created_at: Some(now - Duration::days(24)),
            accessed_at: Some(now - Duration::days(18)),
            modified_at: Some(now - Duration::days(18)),
            recreate_cmd: "cargo build".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\web-dashboard\node_modules"),
            project_name: "web-dashboard".to_string(),
            category: "Node.js / JS".to_string(),
            pattern: "node_modules".to_string(),
            size_bytes: 1_503_238_553, // 1.4 GB
            file_count: 28410,
            created_at: Some(now - Duration::days(5)),
            accessed_at: Some(now - Duration::hours(8)),
            modified_at: Some(now - Duration::hours(8)),
            recreate_cmd: "npm install".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\neural-service\target"),
            project_name: "neural-service".to_string(),
            category: "Rust".to_string(),
            pattern: "target".to_string(),
            size_bytes: 1_181_116_006, // 1.1 GB
            file_count: 1890,
            created_at: Some(now - Duration::days(12)),
            accessed_at: Some(now - Duration::days(3)),
            modified_at: Some(now - Duration::days(3)),
            recreate_cmd: "cargo build".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\mobile-app\android\build"),
            project_name: "mobile-app".to_string(),
            category: "Gradle".to_string(),
            pattern: "build".to_string(),
            size_bytes: 881_852_416, // 841 MB
            file_count: 940,
            created_at: Some(now - Duration::days(28)),
            accessed_at: Some(now - Duration::days(16)),
            modified_at: Some(now - Duration::days(16)),
            recreate_cmd: "gradle build".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\cloud-backend\bin"),
            project_name: "cloud-backend".to_string(),
            category: ".NET / C#".to_string(),
            pattern: "bin".to_string(),
            size_bytes: 536_870_912, // 512 MB
            file_count: 320,
            created_at: Some(now - Duration::days(15)),
            accessed_at: Some(now - Duration::days(4)),
            modified_at: Some(now - Duration::days(4)),
            recreate_cmd: "dotnet build".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\analytics-portal\.next"),
            project_name: "analytics-portal".to_string(),
            category: "Next.js".to_string(),
            pattern: ".next".to_string(),
            size_bytes: 403_914_752, // 385 MB
            file_count: 1420,
            created_at: Some(now - Duration::days(10)),
            accessed_at: Some(now - Duration::days(1)),
            modified_at: Some(now - Duration::days(1)),
            recreate_cmd: "npm run build".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\ai-pipeline\.venv"),
            project_name: "ai-pipeline".to_string(),
            category: "Python".to_string(),
            pattern: ".venv".to_string(),
            size_bytes: 278_921_216, // 266 MB
            file_count: 4200,
            created_at: Some(now - Duration::days(32)),
            accessed_at: Some(now - Duration::days(6)),
            modified_at: Some(now - Duration::days(6)),
            recreate_cmd: "python -m venv .venv && pip install -r requirements.txt".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\docs-site\dist"),
            project_name: "docs-site".to_string(),
            category: "Bundlers".to_string(),
            pattern: "dist".to_string(),
            size_bytes: 149_946_368, // 143 MB
            file_count: 380,
            created_at: Some(now - Duration::days(40)),
            accessed_at: Some(now - Duration::days(9)),
            modified_at: Some(now - Duration::days(9)),
            recreate_cmd: "npm run build".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\api-gateway\target"),
            project_name: "api-gateway".to_string(),
            category: "Rust".to_string(),
            pattern: "target".to_string(),
            size_bytes: 103_809_024, // 99 MB
            file_count: 240,
            created_at: Some(now - Duration::days(2)),
            accessed_at: Some(now - Duration::hours(5)),
            modified_at: Some(now - Duration::hours(5)),
            recreate_cmd: "cargo build".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\ai-pipeline\src\__pycache__"),
            project_name: "ai-pipeline".to_string(),
            category: "Python".to_string(),
            pattern: "__pycache__".to_string(),
            size_bytes: 1_258_291, // 1.2 MB
            file_count: 24,
            created_at: Some(now - Duration::days(6)),
            accessed_at: Some(now - Duration::days(6)),
            modified_at: Some(now - Duration::days(6)),
            recreate_cmd: "python -m py_compile".to_string(),
            files: None,
        },
        ArtifactInfo {
            path: PathBuf::from(r"C:\Developer\Projects\ai-pipeline\.pytest_cache"),
            project_name: "ai-pipeline".to_string(),
            category: "Python".to_string(),
            pattern: ".pytest_cache".to_string(),
            size_bytes: 430_080, // 420 KB
            file_count: 12,
            created_at: Some(now - Duration::days(6)),
            accessed_at: Some(now - Duration::days(6)),
            modified_at: Some(now - Duration::days(6)),
            recreate_cmd: "pytest".to_string(),
            files: None,
        },
    ];

    ui::print_banner();
    println!("\nScanning workspace: C:\\Developer\\Projects");
    ui::display_artifacts_table_limited(&items, Some(30));
    println!("Scan time = {}", "42ms".bold().cyan());
}
