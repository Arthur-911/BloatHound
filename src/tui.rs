use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::collections::BTreeSet;
use std::io::{stdout, Write};

use crate::analyzer::analyze_folder_integrity;
use crate::explorer::open_in_file_explorer;
use crate::models::{format_bytes, format_time_relative, ArtifactInfo};

struct RawModeGuard;

impl RawModeGuard {
    fn enter() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = execute!(out, cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

struct TuiItem {
    artifact: ArtifactInfo,
    selected: bool,
    is_safe: bool,
    verdict_badge: String,
    verdict_detail: String,
    relative_time: String,
    size_str: String,
}

pub fn run_interactive_navigator(artifacts: &[ArtifactInfo]) -> Option<Vec<ArtifactInfo>> {
    if artifacts.is_empty() {
        return None;
    }

    let _guard = RawModeGuard::enter().ok()?;

    // Drain any residual key events from the input buffer
    while event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
        let _ = event::read();
    }

    // Pre-calculate analysis and display models once
    let mut items: Vec<TuiItem> = artifacts
        .iter()
        .map(|art| {
            let analysis = analyze_folder_integrity(art);
            let is_safe = !analysis.is_needed;
            let verdict_badge = if is_safe {
                "[SAFE TO DELETE]".to_string()
            } else {
                "[MATTERS - KEEP]".to_string()
            };
            let relative_time = format_time_relative(art.modified_at);
            let size_str = format_bytes(art.size_bytes);

            TuiItem {
                artifact: art.clone(),
                selected: is_safe,
                is_safe,
                verdict_badge,
                verdict_detail: analysis.verdict_summary,
                relative_time,
                size_str,
            }
        })
        .collect();

    // Collect unique categories for filtering tabs
    let mut category_set = BTreeSet::new();
    for item in &items {
        category_set.insert(item.artifact.category.clone());
    }
    let mut categories = vec!["All".to_string()];
    categories.extend(category_set);

    let mut current_cat_idx: usize = 0;
    let mut cursor_idx: usize = 0;
    let mut scroll_offset: usize = 0;
    let mut status_msg: Option<String> = None;

    loop {
        // Filter items based on active category
        let active_cat = &categories[current_cat_idx];
        let filtered_indices: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| active_cat == "All" || &item.artifact.category == active_cat)
            .map(|(idx, _)| idx)
            .collect();

        if cursor_idx >= filtered_indices.len() && !filtered_indices.is_empty() {
            cursor_idx = filtered_indices.len() - 1;
        }

        let (term_width, term_height) = terminal::size().unwrap_or((100, 28));
        let width = (term_width as usize).max(70);
        let height = term_height as usize;

        let mut out = stdout();
        let _ = execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All));

        // 1. Header Banner
        let _ = execute!(
            out,
            SetForegroundColor(Color::Cyan),
            Print("BloatHound Interactive Navigator\r\n"),
            ResetColor,
        );

        // 2. Category Tabs Bar
        let _ = execute!(out, SetForegroundColor(Color::DarkGrey), Print("Categories: "), ResetColor);
        for (cat_i, cat_name) in categories.iter().enumerate() {
            let count = if cat_name == "All" {
                items.len()
            } else {
                items.iter().filter(|i| &i.artifact.category == cat_name).count()
            };

            let is_active = cat_i == current_cat_idx;
            if is_active {
                let _ = execute!(
                    out,
                    SetForegroundColor(Color::Black),
                    SetBackgroundColor(Color::Cyan),
                    Print(format!(" [{}] {} ({}) ", cat_i + 1, cat_name, count)),
                    ResetColor,
                    Print(" ")
                );
            } else {
                let _ = execute!(
                    out,
                    SetForegroundColor(Color::Grey),
                    Print(format!("[{}] {} ({}) ", cat_i + 1, cat_name, count)),
                    ResetColor
                );
            }
        }
        let _ = execute!(out, Print("\r\n\r\n"));

        // Column widths
        let col_sel = 5;
        let col_status = 18;
        let col_proj = 16;
        let col_cat = 14;
        let col_size = 10;
        let col_act = 11;
        let fixed_cols = col_sel + col_status + col_proj + col_cat + col_size + col_act + 8; // separators
        let col_path = if width > fixed_cols + 10 {
            width - fixed_cols - 2
        } else {
            20
        };

        // Table Borders
        let _ = execute!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "╭{}┬{}┬{}┬{}┬{}┬{}┬{}╮\r\n",
                "─".repeat(col_sel),
                "─".repeat(col_status),
                "─".repeat(col_proj),
                "─".repeat(col_cat),
                "─".repeat(col_size),
                "─".repeat(col_act),
                "─".repeat(col_path)
            )),
            ResetColor,
            SetForegroundColor(Color::White),
            Print(format!(
                "│ {:<3} │ {:<16} │ {:<14} │ {:<12} │ {:>8} │ {:<9} │ {:<width$} │\r\n",
                "Sel",
                "Status / Verdict",
                "Project",
                "Category",
                "Size",
                "Active",
                "Artifact Path",
                width = col_path
            )),
            ResetColor,
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "╞{}╪{}╪{}╪{}╪{}╪{}╪{}╡\r\n",
                "═".repeat(col_sel),
                "═".repeat(col_status),
                "═".repeat(col_proj),
                "═".repeat(col_cat),
                "═".repeat(col_size),
                "═".repeat(col_act),
                "═".repeat(col_path)
            )),
            ResetColor,
        );

        // Viewport height calculation
        let non_table_lines = 14;
        let visible_rows = if height > non_table_lines {
            height - non_table_lines
        } else {
            4
        };

        if cursor_idx >= scroll_offset + visible_rows {
            scroll_offset = cursor_idx.saturating_sub(visible_rows).saturating_add(1);
        } else if cursor_idx < scroll_offset {
            scroll_offset = cursor_idx;
        }

        let end_idx = (scroll_offset + visible_rows).min(filtered_indices.len());

        if filtered_indices.is_empty() {
            let empty_msg = "  No artifacts found matching this category filter.";
            let _ = execute!(
                out,
                SetForegroundColor(Color::DarkGrey),
                Print(format!("│ {:<width$} │\r\n", empty_msg, width = width - 4)),
                ResetColor
            );
        } else {
            for (v_idx, &real_idx) in filtered_indices[scroll_offset..end_idx].iter().enumerate() {
                let display_row_idx = scroll_offset + v_idx;
                let is_highlighted = display_row_idx == cursor_idx;
                let item = &items[real_idx];

                let sel_marker = if item.selected { "[X]" } else { "[ ]" };
                let pointer = if is_highlighted { ">" } else { " " };
                let sel_str = format!("{}{}", pointer, sel_marker);

                let trunc_proj = truncate_str(&item.artifact.project_name, col_proj);
                let trunc_cat = truncate_str(&item.artifact.category, col_cat);
                let trunc_path = truncate_str(&item.artifact.path.display().to_string(), col_path);

                if is_highlighted {
                    let _ = execute!(
                        out,
                        SetBackgroundColor(Color::DarkBlue),
                        SetForegroundColor(Color::White)
                    );
                }

                let _ = execute!(
                    out,
                    Print(format!(
                        "│ {:<col_sel$} │ {:<col_status$} │ {:<col_proj$} │ {:<col_cat$} │ {:>col_size$} │ {:<col_act$} │ {:<col_path$} │\r\n",
                        sel_str,
                        item.verdict_badge,
                        trunc_proj,
                        trunc_cat,
                        item.size_str,
                        item.relative_time,
                        trunc_path,
                        col_sel = col_sel,
                        col_status = col_status,
                        col_proj = col_proj,
                        col_cat = col_cat,
                        col_size = col_size,
                        col_act = col_act,
                        col_path = col_path
                    )),
                    ResetColor
                );
            }
        }

        // Table bottom border
        let _ = execute!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "╰{}┴{}┴{}┴{}┴{}┴{}┴{}╯\r\n",
                "─".repeat(col_sel),
                "─".repeat(col_status),
                "─".repeat(col_proj),
                "─".repeat(col_cat),
                "─".repeat(col_size),
                "─".repeat(col_act),
                "─".repeat(col_path)
            )),
            ResetColor
        );

        // 3. Highlighted Item Details Card
        if let Some(&real_idx) = filtered_indices.get(cursor_idx) {
            let item = &items[real_idx];
            let card_inner_width = width.saturating_sub(4).max(40);
            let p_str = truncate_str(&item.artifact.path.display().to_string(), card_inner_width.saturating_sub(10));
            let v_str = truncate_str(&item.verdict_detail, card_inner_width.saturating_sub(12));
            let r_str = format!("Run `{}` to regenerate if ever needed", item.artifact.recreate_cmd);
            let r_trunc = truncate_str(&r_str, card_inner_width.saturating_sub(12));

            let _ = execute!(
                out,
                SetForegroundColor(Color::Cyan),
                Print(format!("╭─ Target Details ─{}╮\r\n", "─".repeat(width.saturating_sub(21)))),
                Print(format!("│ Exact Path:  {:<width$} │\r\n", p_str, width = card_inner_width.saturating_sub(1))),
                Print(format!("│ Assessment:  {:<width$} │\r\n", v_str, width = card_inner_width.saturating_sub(1))),
                Print(format!("│ Restore Cmd: {:<width$} │\r\n", r_trunc, width = card_inner_width.saturating_sub(1))),
                Print(format!("╰──────────────────{}╯\r\n", "─".repeat(width.saturating_sub(21)))),
                ResetColor
            );
        }

        // 4. Metrics & Status
        let selected_count = items.iter().filter(|i| i.selected).count();
        let selected_bytes: u64 = items
            .iter()
            .filter(|i| i.selected)
            .map(|i| i.artifact.size_bytes)
            .sum();

        if let Some(ref msg) = status_msg {
            let _ = execute!(out, SetForegroundColor(Color::Green), Print(format!("Notice: {}\r\n", msg)), ResetColor);
        } else {
            let summary = format!(
                "Selected: {}/{} targets ({} to reclaim)  |  Viewing category: {}",
                selected_count,
                items.len(),
                format_bytes(selected_bytes),
                active_cat
            );
            let _ = execute!(out, SetForegroundColor(Color::Yellow), Print(format!("{}\r\n", summary)), ResetColor);
        }

        // 5. Controls Guide
        let _ = execute!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print("Controls: [Up/Down/k/j] Move  |  [Space] Select  |  [A] Select All Safe  |  [Tab/1-9] Category  |  [O] Open  |  [Enter] Purge  |  [Q] Exit"),
            ResetColor
        );

        let _ = out.flush();

        // Key Event Handling
        if let Ok(Event::Key(key_event)) = event::read() {
            if key_event.kind == KeyEventKind::Release {
                continue;
            }

            status_msg = None;

            match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    cursor_idx = cursor_idx.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !filtered_indices.is_empty() && cursor_idx + 1 < filtered_indices.len() {
                        cursor_idx += 1;
                    }
                }
                KeyCode::Home => {
                    cursor_idx = 0;
                }
                KeyCode::End => {
                    if !filtered_indices.is_empty() {
                        cursor_idx = filtered_indices.len() - 1;
                    }
                }
                KeyCode::Tab => {
                    current_cat_idx = (current_cat_idx + 1) % categories.len();
                    cursor_idx = 0;
                    scroll_offset = 0;
                }
                KeyCode::BackTab => {
                    current_cat_idx = if current_cat_idx == 0 {
                        categories.len() - 1
                    } else {
                        current_cat_idx - 1
                    };
                    cursor_idx = 0;
                    scroll_offset = 0;
                }
                KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                    let num = (c as u8 - b'1') as usize;
                    if num < categories.len() {
                        current_cat_idx = num;
                        cursor_idx = 0;
                        scroll_offset = 0;
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(&real_idx) = filtered_indices.get(cursor_idx) {
                        if let Some(item) = items.get_mut(real_idx) {
                            item.selected = !item.selected;
                        }
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    let any_safe_unselected = filtered_indices
                        .iter()
                        .filter_map(|&r| items.get(r))
                        .any(|i| i.is_safe && !i.selected);

                    for &real_idx in &filtered_indices {
                        if let Some(item) = items.get_mut(real_idx) {
                            if item.is_safe {
                                item.selected = any_safe_unselected;
                            }
                        }
                    }
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    if let Some(&real_idx) = filtered_indices.get(cursor_idx) {
                        if let Some(item) = items.get(real_idx) {
                            let path = &item.artifact.path;
                            match open_in_file_explorer(path) {
                                Ok(_) => {
                                    status_msg = Some(format!("Opened in File Explorer: {}", path.display()));
                                }
                                Err(e) => {
                                    status_msg = Some(format!("Error opening folder: {}", e));
                                }
                            }
                        }
                    }
                }
                KeyCode::Enter => {
                    let selected_artifacts: Vec<ArtifactInfo> = items
                        .into_iter()
                        .filter(|i| i.selected)
                        .map(|i| i.artifact)
                        .collect();
                    return Some(selected_artifacts);
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    return None;
                }
                _ => {}
            }
        }
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}
