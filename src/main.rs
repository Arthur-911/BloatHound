use clap::{Parser, Subcommand};
use bloathound::{analyzer, cleaner, explorer, inspector, models::format_bytes, scanner, tui, ui};
use dialoguer::Confirm;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "bloathound",
    author = "Arthur <https://github.com/Arthur-911>",
    version = "0.1.0",
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

        /// Launch interactive TUI navigator (arrow keys, space to select, 'o' to open)
        #[arg(short, long)]
        interactive: bool,
    },

    /// Launch interactive keyboard navigator (arrow keys, space, 'o' to open)
    #[command(alias = "nav", alias = "tui")]
    Interactive {
        /// Target directory to scan (defaults to current folder)
        #[arg(default_value = ".")]
        path: PathBuf,
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

        /// Launch interactive TUI selector to pick targets with checkboxes
        #[arg(short, long)]
        interactive: bool,
    },
}

fn main() {
    ui::print_banner();

    let cli = Cli::parse();

    // Default action: if no subcommand is given, immediately scan the current folder!
    let command = cli.command.unwrap_or(Commands::Scan {
        path: PathBuf::from("."),
        detailed: false,
        max_files: 15,
        open: None,
        interactive: false,
    });

    match command {
        Commands::Scan {
            path,
            detailed,
            max_files,
            open,
            interactive,
        } => {
            println!("\nScanning workspace: {}", path.display());

            match scanner::scan_directory(&path, 10) {
                Ok(artifacts) => {
                    if artifacts.is_empty() {
                        println!("\nAll clean! No disposable artifacts found in target path.");
                        return;
                    }

                    if interactive {
                        if let Some(selected) = tui::run_interactive_navigator(&artifacts) {
                            if selected.is_empty() {
                                println!("\nNo folders were selected for purging.");
                            } else {
                                let total_bytes: u64 = selected.iter().map(|a| a.size_bytes).sum();
                                println!(
                                    "\nSelected {} folders ({}) for cleanup.",
                                    selected.len(),
                                    format_bytes(total_bytes)
                                );
                                let proceed = Confirm::new()
                                    .with_prompt("Proceed with deletion of selected folders?")
                                    .default(false)
                                    .interact()
                                    .unwrap_or(false);
                                if proceed {
                                    let result = cleaner::purge_artifacts(&selected, false);
                                    ui::display_clean_summary(&result);
                                } else {
                                    println!("\nCleanup cancelled. No files were deleted.");
                                }
                            }
                        }
                        return;
                    }

                    ui::display_artifacts_table(&artifacts);

                    if detailed {
                        for art in &artifacts {
                            let files = inspector::list_files_with_timestamps(&art.path, max_files);
                            let title = format!("{} -> {}", art.project_name, art.pattern);
                            ui::display_files_table(&files, &title);
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
                        // Interactive jump or launch prompt
                        print!(
                            "\nEnter folder # to open [1-{}], 'i' for interactive navigator, or press Enter to skip: ",
                            artifacts.len()
                        );
                        let _ = std::io::stdout().flush();
                        let mut input = String::new();
                        if std::io::stdin().read_line(&mut input).is_ok() {
                            let trimmed = input.trim();
                            if trimmed.eq_ignore_ascii_case("i") {
                                if let Some(selected) = tui::run_interactive_navigator(&artifacts) {
                                    if !selected.is_empty() {
                                        let total_bytes: u64 = selected.iter().map(|a| a.size_bytes).sum();
                                        println!(
                                            "\nSelected {} folders ({}) for cleanup.",
                                            selected.len(),
                                            format_bytes(total_bytes)
                                        );
                                        let proceed = Confirm::new()
                                            .with_prompt("Proceed with deletion of selected folders?")
                                            .default(false)
                                            .interact()
                                            .unwrap_or(false);
                                        if proceed {
                                            let result = cleaner::purge_artifacts(&selected, false);
                                            ui::display_clean_summary(&result);
                                        } else {
                                            println!("\nCleanup cancelled. No files were deleted.");
                                        }
                                    }
                                }
                            } else if let Ok(idx) = trimmed.parse::<usize>() {
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

        Commands::Interactive { path } => {
            println!("\nScanning workspace: {}", path.display());
            match scanner::scan_directory(&path, 10) {
                Ok(artifacts) => {
                    if artifacts.is_empty() {
                        println!("\nAll clean! No disposable artifacts found in target path.");
                        return;
                    }
                    if let Some(selected) = tui::run_interactive_navigator(&artifacts) {
                        if selected.is_empty() {
                            println!("\nNo folders were selected for purging.");
                        } else {
                            let total_bytes: u64 = selected.iter().map(|a| a.size_bytes).sum();
                            println!(
                                "\nSelected {} folders ({}) for cleanup.",
                                selected.len(),
                                format_bytes(total_bytes)
                            );
                            let proceed = Confirm::new()
                                .with_prompt("Proceed with deletion of selected folders?")
                                .default(false)
                                .interact()
                                .unwrap_or(false);
                            if proceed {
                                let result = cleaner::purge_artifacts(&selected, false);
                                ui::display_clean_summary(&result);
                            } else {
                                println!("\nCleanup cancelled. No files were deleted.");
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

        Commands::Analyze { path, yes } => {
            println!("\nAnalyzing workspace for project connections and deletion impact: {}", path.display());

            let artifacts = match scanner::scan_directory(&path, 10) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("\nError scanning path: {}", e);
                    std::process::exit(1);
                }
            };

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
            interactive,
        } => {
            println!("\nScanning workspace for purgeable folders: {}", path.display());

            let artifacts = match scanner::scan_directory(&path, 10) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("\nError scanning path: {}", e);
                    std::process::exit(1);
                }
            };

            if artifacts.is_empty() {
                println!("\nAll clean! No disposable artifacts found in target path.");
                return;
            }

            let targets_to_clean = if interactive {
                match tui::run_interactive_navigator(&artifacts) {
                    Some(selected) => {
                        if selected.is_empty() {
                            println!("\nNo folders were selected for purging.");
                            return;
                        }
                        selected
                    }
                    None => {
                        println!("\nInteractive selection cancelled. No files were deleted.");
                        return;
                    }
                }
            } else {
                ui::display_artifacts_table(&artifacts);
                artifacts
            };

            if !yes && !dry_run {
                let confirmed = Confirm::new()
                    .with_prompt(format!(
                        "\nAre you sure you want to permanently delete these {} artifact folders?",
                        targets_to_clean.len()
                    ))
                    .default(false)
                    .interact()
                    .unwrap_or(false);

                if !confirmed {
                    println!("\nCleanup cancelled. No files were deleted.");
                    return;
                }
            }

            let result = cleaner::purge_artifacts(&targets_to_clean, dry_run);
            ui::display_clean_summary(&result);
        }
    }
}
