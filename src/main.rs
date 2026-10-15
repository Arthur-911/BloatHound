use clap::{Parser, Subcommand};
use bloathound::{analyzer, cleaner, explorer, inspector, scanner, ui};
use dialoguer::Confirm;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "bloathound",
    author = "Arthur <https://github.com/Arthur-911>",
    version = "0.2.3",
    about = "Lightning-fast, safety-first developer artifact and cache purger with file lifecycle analytics."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a directory for disposable build artifacts (default when run without arguments)
    Scan {
        /// Target directory to scan (defaults to current folder)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Display individual file breakdown with made, opened, and modified timestamps
        #[arg(short, long)]
        detailed: bool,

        /// Maximum files to display per artifact in detailed view
        #[arg(long, default_value = "15")]
        max_files: usize,

        /// Automatically open the Nth artifact folder in File Explorer
        #[arg(short, long)]
        open: Option<usize>,

        /// Output scan results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Deep-inspect a specific folder to view files with timestamps
    Inspect {
        /// Folder path to inspect
        path: PathBuf,

        /// Maximum number of files to show
        #[arg(long, default_value = "25")]
        limit: usize,
    },

    /// Open a specific folder directly in Windows File Explorer
    Open {
        /// Folder path to open
        path: PathBuf,
    },

    /// Analyze whether folders are connected to project files and if deleting matters
    #[command(alias = "check", alias = "audit")]
    Analyze {
        /// Target directory to analyze (defaults to current folder)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Automatically purge all folders that do not matter without asking
        #[arg(short, long)]
        yes: bool,

        /// Output analysis results in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Purge disposable artifact directories to reclaim disk space
    Clean {
        /// Target directory to scan and clean (defaults to current folder)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Simulate cleanup without actually deleting any files
        #[arg(long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Default action: if no subcommand is given, immediately scan the current folder!
    let command = cli.command.unwrap_or(Commands::Scan {
        path: PathBuf::from("."),
        detailed: false,
        max_files: 15,
        open: None,
        json: false,
    });

    let is_json = matches!(
        &command,
        Commands::Scan { json: true, .. } | Commands::Analyze { json: true, .. }
    );

    if !is_json {
        ui::print_banner();
    }

    match command {
        Commands::Scan {
            path,
            detailed,
            max_files,
            open,
            json,
        } => {
            if !json {
                println!("\nScanning workspace: {}", path.display());
            }

            match scanner::scan_directory(&path, 10) {
                Ok(mut artifacts) => {
                    if detailed {
                        for art in &mut artifacts {
                            let files = inspector::list_files_with_timestamps(&art.path, max_files);
                            art.files = Some(files);
                        }
                    }

                    if json {
                        let total_bytes: u64 = artifacts.iter().map(|a| a.size_bytes).sum();
                        let report = bloathound::models::ScanReport {
                            summary: bloathound::models::ScanSummary {
                                total_artifacts: artifacts.len(),
                                total_bytes,
                                human_total_size: bloathound::models::format_bytes(total_bytes),
                            },
                            artifacts,
                        };
                        match serde_json::to_string_pretty(&report) {
                            Ok(json_output) => println!("{}", json_output),
                            Err(e) => {
                                eprintln!("Error serializing scan report: {}", e);
                                std::process::exit(1);
                            }
                        }
                        return;
                    }

                    if artifacts.is_empty() {
                        println!("\nAll clean! No disposable artifacts found in target path.");
                        return;
                    }

                    ui::display_artifacts_table(&artifacts);

                    if detailed {
                        for art in &artifacts {
                            if let Some(files) = &art.files {
                                let title = format!("{} -> {}", art.project_name, art.pattern);
                                ui::display_files_table(files, &title);
                            }
                        }
                    }

                    if let Some(target_idx) = open {
                        if target_idx >= 1 && target_idx <= artifacts.len() {
                            let target_path = &artifacts[target_idx - 1].path;
                            println!("\nOpening in File Explorer: {}", target_path.display());
                            if let Err(e) = explorer::open_in_file_explorer(target_path) {
                                eprintln!("Error opening folder: {}", e);
                            }
                        } else {
                            eprintln!(
                                "\nInvalid folder number {}. Please choose between 1 and {}.",
                                target_idx,
                                artifacts.len()
                            );
                        }
                    } else {
                        // Interactive jump prompt
                        print!(
                            "\nEnter folder # to open directly in File Explorer [1-{}] (or press Enter to skip): ",
                            artifacts.len()
                        );
                        let _ = std::io::stdout().flush();
                        let mut input = String::new();
                        if std::io::stdin().read_line(&mut input).is_ok() {
                            let trimmed = input.trim();
                            if let Ok(idx) = trimmed.parse::<usize>() {
                                if idx >= 1 && idx <= artifacts.len() {
                                    let target_path = &artifacts[idx - 1].path;
                                    println!("Opening in File Explorer: {}", target_path.display());
                                    if let Err(e) = explorer::open_in_file_explorer(target_path) {
                                        eprintln!("Error opening folder: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\nError scanning path: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Inspect { path, limit } => {
            if !path.exists() {
                eprintln!("\nError: Path does not exist: {}", path.display());
                std::process::exit(1);
            }

            println!("\nInspecting folder: {}", path.display());
            let files = inspector::list_files_with_timestamps(&path, limit);
            let folder_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("target");
            ui::display_files_table(&files, folder_name);

            let confirmed = Confirm::new()
                .with_prompt("\nOpen this folder in File Explorer?")
                .default(false)
                .interact()
                .unwrap_or(false);

            if confirmed {
                if let Err(e) = explorer::open_in_file_explorer(&path) {
                    eprintln!("Error: {}", e);
                }
            }
        }

        Commands::Open { path } => {
            println!("\nOpening in File Explorer: {}", path.display());
            if let Err(e) = explorer::open_in_file_explorer(&path) {
                eprintln!("\nError: {}", e);
                std::process::exit(1);
            }
        }

        Commands::Analyze { path, yes, json } => {
            if !json {
                println!("\nAnalyzing workspace for project connections and deletion impact: {}", path.display());
            }

            let artifacts = match scanner::scan_directory(&path, 10) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("\nError scanning path: {}", e);
                    std::process::exit(1);
                }
            };

            if json {
                let report = analyzer::AnalysisReport::from_artifacts(&artifacts);
                match serde_json::to_string_pretty(&report) {
                    Ok(json_output) => println!("{}", json_output),
                    Err(e) => {
                        eprintln!("Error serializing analysis report: {}", e);
                        std::process::exit(1);
                    }
                }
                return;
            }

            if artifacts.is_empty() {
                println!("\nAll clean! No artifacts found.");
                return;
            }

            ui::display_integrity_analysis_table(&artifacts);

            let does_not_matter: Vec<_> = artifacts
                .into_iter()
                .filter(|a| !analyzer::analyze_folder_integrity(a).is_needed)
                .collect();

            if does_not_matter.is_empty() {
                println!("\nCaution: All detected folders are connected/needed. None are safe to delete.");
                return;
            }

            let proceed = if yes {
                true
            } else {
                Confirm::new()
                    .with_prompt(format!(
                        "\nWould you like to safely purge all {} folders that DO NOT MATTER?",
                        does_not_matter.len()
                    ))
                    .default(false)
                    .interact()
                    .unwrap_or(false)
            };

            if proceed {
                let result = cleaner::purge_artifacts(&does_not_matter, false);
                ui::display_clean_summary(&result);
            } else {
                println!("\nNo files were deleted.");
            }
        }

        Commands::Clean {
            path,
            dry_run,
            yes,
        } => {
            println!("\nScanning workspace for purgeable folders: {}", path.display());

            let artifacts = match scanner::scan_directory(&path, 10) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("\nError scanning path: {}", e);
                    std::process::exit(1);
                }
            };

            ui::display_artifacts_table(&artifacts);

            if artifacts.is_empty() {
                println!("\nAll clean! No disposable artifacts found in target path.");
                return;
            }

            if !yes && !dry_run {
                let confirmed = Confirm::new()
                    .with_prompt(format!(
                        "\nAre you sure you want to permanently delete these {} artifact folders?",
                        artifacts.len()
                    ))
                    .default(false)
                    .interact()
                    .unwrap_or(false);

                if !confirmed {
                    println!("\nCleanup cancelled. No files were deleted.");
                    return;
                }
            }

            let result = cleaner::purge_artifacts(&artifacts, dry_run);
            ui::display_clean_summary(&result);
        }
    }
}
