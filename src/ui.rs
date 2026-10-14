use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};
use colored::*;

use crate::analyzer::analyze_folder_integrity;
use crate::cleaner::CleanResult;
use crate::models::{format_bytes, ArtifactInfo, FileDetail, Verdict};

pub fn print_banner() {
    println!("{}", "===================================================".cyan());
    println!("{}", "               >> BloatHound <<                    ".bold().cyan());
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
        Cell::new("Status / Verdict").fg(Color::White),
        Cell::new("Project").fg(Color::White),
        Cell::new("Artifact Folder").fg(Color::Yellow),
        Cell::new("Category").fg(Color::Cyan),
        Cell::new("Size").fg(Color::Green).set_alignment(CellAlignment::Right),
        Cell::new("When Made (Created)").fg(Color::Blue),
        Cell::new("Last Opened (Accessed)").fg(Color::Yellow),
        Cell::new("Exact Folder Path").fg(Color::Magenta),
    ]);

    let mut total_bytes: u64 = 0;

    for (idx, item) in artifacts.iter().enumerate() {
        total_bytes += item.size_bytes;

        let (verdict, _) = item.assess_necessity();
        let (status_text, status_color) = match verdict {
            Verdict::NotNeeded => ("[DISPOSABLE]", Color::Red),
            Verdict::Review => ("[REVIEW]", Color::Yellow),
            Verdict::KeepActive => ("[ACTIVE]", Color::Green),
        };

        table.add_row(vec![
            Cell::new((idx + 1).to_string()),
            Cell::new(status_text).fg(status_color),
            Cell::new(&item.project_name),
            Cell::new(&item.pattern).fg(Color::Yellow),
            Cell::new(&item.category).fg(Color::Cyan),
            Cell::new(item.human_size()).fg(Color::Green).set_alignment(CellAlignment::Right),
            Cell::new(item.created_str()).fg(Color::Blue),
            Cell::new(item.accessed_str()).fg(Color::Yellow),
            Cell::new(item.path.display().to_string()).fg(Color::DarkGrey),
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

pub fn display_audit_report(artifacts: &[ArtifactInfo]) {
    println!("\n{}", "===================================================".bold().cyan());
    println!("{}", "           INTELLIGENT NECESSITY AUDIT             ".bold().white());
    println!("{}", "===================================================".bold().cyan());

    let mut not_needed = Vec::new();
    let mut review = Vec::new();
    let mut keep = Vec::new();

    let mut not_needed_bytes: u64 = 0;

    for item in artifacts {
        let (verdict, _) = item.assess_necessity();
        match verdict {
            Verdict::NotNeeded => {
                not_needed_bytes += item.size_bytes;
                not_needed.push(item);
            }
            Verdict::Review => review.push(item),
            Verdict::KeepActive => keep.push(item),
        }
    }

    // 1. NOT NEEDED
    println!(
        "\n{}",
        format!(
            "DISPOSABLE - INACTIVE ({} folders, {} reclaimable):",
            not_needed.len(),
            format_bytes(not_needed_bytes)
        )
        .bold()
        .red()
    );

    if not_needed.is_empty() {
        println!("  None. All detected artifacts are active or under review.");
    } else {
        for (idx, item) in not_needed.iter().enumerate() {
            let (_, reason) = item.assess_necessity();
            println!(
                "  [{}] {} ({}) - {}",
                (idx + 1).to_string().bold(),
                item.project_name.bold().white(),
                item.human_size().bold().green(),
                reason.dimmed()
            );
            println!("      Path: {}", item.path.display().to_string().dimmed());
        }
    }

    // 2. REVIEW
    if !review.is_empty() {
        println!(
            "\n{}",
            format!("REVIEW - MODERATE INACTIVITY ({} folders):", review.len())
                .bold()
                .yellow()
        );
        for item in &review {
            let (_, reason) = item.assess_necessity();
            println!(
                "  - {} ({}) - {}",
                item.project_name.bold().white(),
                item.human_size().bold().green(),
                reason.dimmed()
            );
        }
    }

    // 3. KEEP / ACTIVE
    if !keep.is_empty() {
        println!(
            "\n{}",
            format!("KEEP - IN ACTIVE USE ({} folders):", keep.len())
                .bold()
                .green()
        );
        for item in &keep {
            let (_, reason) = item.assess_necessity();
            println!(
                "  - {} ({}) - {}",
                item.project_name.bold().white(),
                item.human_size().bold().green(),
                reason.dimmed()
            );
        }
    }

    println!("\n{}", "---------------------------------------------------".cyan());
    println!(
        "Recommendation: You can immediately free {} by purging {} not-needed folders.",
        format_bytes(not_needed_bytes).bold().green(),
        not_needed.len().to_string().bold().red()
    );
    println!("{}", "---------------------------------------------------".cyan());
}

/// Displays deep structural dependency & project connection analysis:
/// Checks if it's connected to other files, and whether it MATTERS if deleted.
pub fn display_integrity_analysis_table(artifacts: &[ArtifactInfo]) {
    println!("\n{}", "=====================================================================".bold().cyan());
    println!("{}", "       STRUCTURAL INTEGRITY & DEPENDENCY ANALYSIS REPORT             ".bold().white());
    println!("{}", "    (Checks project linkage, custom files, and if deleting matters)   ".dimmed());
    println!("{}", "=====================================================================".bold().cyan());

    if artifacts.is_empty() {
        println!("\nAll clean! No artifacts found to analyze.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        Cell::new("#").fg(Color::DarkGrey),
        Cell::new("Project / Artifact").fg(Color::White),
        Cell::new("Size").fg(Color::Green).set_alignment(CellAlignment::Right),
        Cell::new("Manifest Connection").fg(Color::Cyan),
        Cell::new("Custom Files?").fg(Color::Yellow),
        Cell::new("Imported in Code?").fg(Color::Blue),
        Cell::new("Matters If Deleted?").fg(Color::White),
        Cell::new("Detailed Reason").fg(Color::White),
    ]);

    for (idx, item) in artifacts.iter().enumerate() {
        let analysis = analyze_folder_integrity(item);

        let manifest_text = match &analysis.manifest_file {
            Some(m) => format!("Linked ({})", m),
            None => "No Manifest (Orphan)".to_string(),
        };
        let manifest_color = if analysis.has_manifest { Color::Green } else { Color::DarkGrey };

        let custom_files_text = if analysis.has_custom_user_files {
            format!("{} custom files found", analysis.custom_files_found.len())
        } else {
            "Clean (0 custom files)".to_string()
        };
        let custom_files_color = if analysis.has_custom_user_files { Color::Red } else { Color::Green };

        let source_refs_text = if analysis.is_referenced_in_source {
            format!("Imported in {} files", analysis.source_references_found.len())
        } else {
            "Not imported".to_string()
        };
        let source_refs_color = if analysis.is_referenced_in_source { Color::Yellow } else { Color::Green };

        let (verdict_text, verdict_color) = if analysis.is_needed {
            ("[MATTERS - KEEP]", Color::Red)
        } else {
            ("[SAFE TO DELETE]", Color::Green)
        };

        let label = format!("{} ({})", item.project_name, item.pattern);

        table.add_row(vec![
            Cell::new((idx + 1).to_string()),
            Cell::new(label),
            Cell::new(item.human_size()).fg(Color::Green).set_alignment(CellAlignment::Right),
            Cell::new(manifest_text).fg(manifest_color),
            Cell::new(custom_files_text).fg(custom_files_color),
            Cell::new(source_refs_text).fg(source_refs_color),
            Cell::new(verdict_text).fg(verdict_color),
            Cell::new(analysis.verdict_summary),
        ]);
    }

    println!("\n{}", table);
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
