use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};
use colored::*;

use crate::cleaner::CleanResult;
use crate::models::{format_bytes, ArtifactInfo, FileDetail};

pub fn print_banner() {
    println!("{}", "===================================================".cyan());
    println!("{}", "               >> dev-cleaner <<                   ".bold().cyan());
    println!("{}", "   High-Speed Developer Artifact & Cache Purger    ".dimmed());
    println!("{}", "===================================================".cyan());
}

pub fn display_artifacts_table(artifacts: &[ArtifactInfo]) {
    if artifacts.is_empty() {
        println!("\n{}", "All clean! No disposable artifacts found in target path.".bold().green());
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        Cell::new("#").fg(Color::DarkGrey),
        Cell::new("Project").fg(Color::White),
        Cell::new("Artifact Folder").fg(Color::Yellow),
        Cell::new("Category").fg(Color::Cyan),
        Cell::new("Size").fg(Color::Green).set_alignment(CellAlignment::Right),
        Cell::new("When Made (Created)").fg(Color::Blue),
        Cell::new("Last Opened (Accessed)").fg(Color::Yellow),
        Cell::new("Last Modified").fg(Color::White),
    ]);

    let mut total_bytes: u64 = 0;

    for (idx, item) in artifacts.iter().enumerate() {
        total_bytes += item.size_bytes;
        table.add_row(vec![
            Cell::new((idx + 1).to_string()),
            Cell::new(&item.project_name),
            Cell::new(&item.pattern).fg(Color::Yellow),
            Cell::new(&item.category).fg(Color::Cyan),
            Cell::new(item.human_size()).fg(Color::Green).set_alignment(CellAlignment::Right),
            Cell::new(item.created_str()).fg(Color::Blue),
            Cell::new(item.accessed_str()).fg(Color::Yellow),
            Cell::new(item.modified_str()),
        ]);
    }

    println!("\n{}", table);
    println!(
        "\n{} {} across {} artifact folders.",
        "Total Reclaimable Space:".bold(),
        format_bytes(total_bytes).bold().green(),
        artifacts.len().to_string().bold().cyan()
    );
}

pub fn display_files_table(files: &[FileDetail], folder_name: &str) {
    if files.is_empty() {
        println!("\nNo files found inside {}", folder_name);
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        Cell::new("#").fg(Color::DarkGrey),
        Cell::new("Relative File Path").fg(Color::White),
        Cell::new("Size").fg(Color::Green).set_alignment(CellAlignment::Right),
        Cell::new("When Made (Created)").fg(Color::Blue),
        Cell::new("Last Opened (Accessed)").fg(Color::Yellow),
        Cell::new("Last Modified").fg(Color::White),
    ]);

    for (idx, f) in files.iter().enumerate() {
        table.add_row(vec![
            Cell::new((idx + 1).to_string()),
            Cell::new(&f.relative_path),
            Cell::new(f.human_size()).fg(Color::Green).set_alignment(CellAlignment::Right),
            Cell::new(f.created_str()).fg(Color::Blue),
            Cell::new(f.accessed_str()).fg(Color::Yellow),
            Cell::new(f.modified_str()),
        ]);
    }

    println!("\n{}", format!("Files breakdown for '{}':", folder_name).bold().cyan());
    println!("{}", table);
}

pub fn display_clean_summary(result: &CleanResult) {
    println!("\n{}", "---------------------------------------------------".cyan());
    if result.dry_run {
        println!("{}", "[DRY RUN COMPLETE]".bold().yellow());
        println!("Would have deleted: {} folders", result.success_count.to_string().bold());
        println!("Would have reclaimed: {}", result.human_freed_size().bold().green());
    } else {
        if result.error_count == 0 {
            println!("{}", "[CLEANING COMPLETE]".bold().green());
        } else {
            println!("{}", "[CLEANING FINISHED WITH WARNINGS]".bold().yellow());
        }
        println!("Successfully deleted: {} folders", result.success_count.to_string().bold());
        println!("Total space freed: {}", result.human_freed_size().bold().green());
        if result.error_count > 0 {
            println!("Failed to delete: {} folders", result.error_count.to_string().red());
        }
    }
    println!("{}", "---------------------------------------------------".cyan());

    if !result.errors.is_empty() {
        println!("\n{}", "Errors encountered:".red().bold());
        for err in &result.errors {
            println!("  - {}", err.red());
        }
    }
}
