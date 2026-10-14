use chrono::{DateTime, Local};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    NotNeeded,   // Inactive > 30 days: safe to purge
    Review,      // Inactive 14 - 30 days
    KeepActive,  // Active <= 14 days: needed for active development
}

#[derive(Debug, Clone)]
pub struct ArtifactInfo {
    pub path: PathBuf,
    pub project_name: String,
    pub category: String,
    pub pattern: String,
    pub size_bytes: u64,
    pub file_count: usize,
    pub created_at: Option<DateTime<Local>>,
    pub accessed_at: Option<DateTime<Local>>,
    pub modified_at: Option<DateTime<Local>>,
    pub recreate_cmd: String,
}

#[derive(Debug, Clone)]
pub struct FileDetail {
    pub path: PathBuf,
    pub relative_path: String,
    pub size_bytes: u64,
    pub created_at: Option<DateTime<Local>>,
    pub accessed_at: Option<DateTime<Local>>,
    pub modified_at: Option<DateTime<Local>>,
}

pub fn format_bytes(bytes: u64) -> String {
    let mut b = bytes as f64;
    for unit in ["B", "KB", "MB", "GB", "TB"] {
        if b < 1024.0 || unit == "TB" {
            if unit == "B" || unit == "KB" {
                return format!("{} {}", b as u64, unit);
            }
            return format!("{:.1} {}", b, unit);
        }
        b /= 1024.0;
    }
    format!("{} B", bytes)
}

pub fn format_time_relative(dt_opt: Option<DateTime<Local>>) -> String {
    let dt = match dt_opt {
        Some(t) => t,
        None => return "Unknown".to_string(),
    };

    let now = Local::now();
    let diff = now.signed_duration_since(dt);
    let seconds = diff.num_seconds();

    let rel = if seconds < 0 {
        "just now".to_string()
    } else if seconds < 60 {
        format!("{}s ago", seconds)
    } else if seconds < 3600 {
        format!("{}m ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h ago", seconds / 3600)
    } else if seconds < 86400 * 30 {
        format!("{}d ago", seconds / 86400)
    } else if seconds < 86400 * 365 {
        format!("{}mo ago", seconds / (86400 * 30))
    } else {
        format!("{}y ago", seconds / (86400 * 365))
    };

    format!("{} ({})", dt.format("%Y-%m-%d %H:%M"), rel)
}

impl ArtifactInfo {
    pub fn human_size(&self) -> String {
        format_bytes(self.size_bytes)
    }

    pub fn created_str(&self) -> String {
        format_time_relative(self.created_at)
    }

    pub fn accessed_str(&self) -> String {
        format_time_relative(self.accessed_at)
    }

    pub fn modified_str(&self) -> String {
        format_time_relative(self.modified_at)
    }

    /// Evaluates whether the artifact is currently needed based on recent write/modification activity.
    pub fn assess_necessity(&self) -> (Verdict, String) {
        // Modification time (mtime) is the most reliable indicator of real developer activity.
        // Access times (atime) can be artificially bumped by antivirus, backup tools, or scanners.
        let latest_activity = self.modified_at.or(self.accessed_at);

        match latest_activity {
            Some(dt) => {
                let now = Local::now();
                let days = now.signed_duration_since(dt).num_days();

                if days <= 14 {
                    (
                        Verdict::KeepActive,
                        format!("Active ({}d ago) - Needed for current development.", days),
                    )
                } else if days <= 30 {
                    (
                        Verdict::Review,
                        format!("Inactive for {}d - Review before purging.", days),
                    )
                } else {
                    (
                        Verdict::NotNeeded,
                        format!(
                            "Not needed (untouched for {}d). Safe to purge. Recreate with: '{}'.",
                            days, self.recreate_cmd
                        ),
                    )
                }
            }
            None => (
                Verdict::NotNeeded,
                format!(
                    "Not needed (no modification history found). Recreate with: '{}'.",
                    self.recreate_cmd
                ),
            ),
        }
    }

    pub fn verdict_badge(&self) -> (&'static str, &'static str) {
        let (verdict, _) = self.assess_necessity();
        match verdict {
            Verdict::NotNeeded => ("NOT NEEDED", "red"),
            Verdict::Review => ("REVIEW", "yellow"),
            Verdict::KeepActive => ("KEEP / ACTIVE", "green"),
        }
    }
}

impl FileDetail {
    pub fn human_size(&self) -> String {
        format_bytes(self.size_bytes)
    }

    pub fn created_str(&self) -> String {
        format_time_relative(self.created_at)
    }

    pub fn accessed_str(&self) -> String {
        format_time_relative(self.accessed_at)
    }

    pub fn modified_str(&self) -> String {
        format_time_relative(self.modified_at)
    }
}
