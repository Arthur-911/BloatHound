use dev_cleaner::{cleaner, inspector, scanner, ui};
use clap::{Parser, Subcommand};
use dialoguer::Confirm;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "dev-cleaner",
    author = "Developer <dev@example.com>",
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
    },

    /// Deep-inspect a specific folder to view files with timestamps
    Inspect {
        /// Folder path to inspect
        path: PathBuf,

        /// Maximum number of files to show
        #[arg(long, default_value = "25")]
        limit: usize,
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
    ui::print_banner();

    let cli = Cli::parse();

    // Default action: if no subcommand is given, immediately scan the current folder!
    let command = cli.command.unwrap_or(Commands::Scan {
        path: PathBuf::from("."),
        detailed: false,
        max_files: 15,
    });

    match command {
        Commands::Scan {
            path,
            detailed,
            max_files,
        } => {
            println!("\nScanning workspace: {}", path.display());

            match scanner::scan_directory(&path, 10) {
                Ok(artifacts) => {
                    ui::display_artifacts_table(&artifacts);

                    if detailed && !artifacts.is_empty() {
                        for art in &artifacts {
                            let files = inspector::list_files_with_timestamps(&art.path, max_files);
                            let title = format!("{} -> {}", art.project_name, art.pattern);
                            ui::display_files_table(&files, &title);
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
        }

        Commands::Clean { path, dry_run, yes } => {
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
