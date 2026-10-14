use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal,
};
use std::collections::BTreeSet;
use std::fmt::Write as FmtWrite;
use std::io::{stdout, Write};

use crate::analyzer::analyze_folder_integrity;
use crate::explorer::open_in_file_explorer;
use crate::models::{format_bytes, format_time_relative, ArtifactInfo};

struct RawModeGuard;

impl RawModeGuard {
    fn enter() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, terminal::EnterAlternateScreen, cursor::Hide, EnableMouseCapture)?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = execute!(out, cursor::Show, DisableMouseCapture, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

struct TuiItem {
    artifact: ArtifactInfo,
    selected: bool,
    is_safe: bool,
    verdict_badge: &'static str,
    verdict_detail: String,
    relative_time: String,
    size_str: String,
}

pub fn run_interactive_navigator(artifacts: &[ArtifactInfo]) -> Option<Vec<ArtifactInfo>> {
    if artifacts.is_empty() {
        return None;
    }

    let _guard = RawModeGuard::enter().ok()?;

    // Drain any leftover keys from the input buffer
    while event::poll(std::time::Duration::from_millis(30)).unwrap_or(false) {
        let _ = event::read();
    }

    // Pre-calculate analysis and models once for fast 0ms rendering
    let mut items: Vec<TuiItem> = artifacts
        .iter()
        .map(|art| {
            let analysis = analyze_folder_integrity(art);
            let is_safe = !analysis.is_needed;
            let verdict_badge = if is_safe {
                "[SAFE]"
            } else {
                "[KEEP]"
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

    // Unique category tabs
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

        let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
        let width = (term_w as usize).max(60);
        let height = term_h as usize;

        // Build entire screen into a single string buffer for 0ms flicker-free render
        let mut buf = String::with_capacity(8192);

        // Move cursor to home (0, 0)
        buf.push_str("\x1b[H");

        // 1. Header Banner
        buf.push_str("\x1b[1;36mBloatHound Interactive Navigator\x1b[0m\r\n");

        // 2. Category Tabs
        buf.push_str("\x1b[90mCategories: \x1b[0m");
        for (cat_i, cat_name) in categories.iter().enumerate() {
            let count = if cat_name == "All" {
                items.len()
            } else {
                items.iter().filter(|i| &i.artifact.category == cat_name).count()
            };

            let is_active = cat_i == current_cat_idx;
            if is_active {
                let _ = write!(buf, "\x1b[30;46m [{}] {} ({}) \x1b[0m ", cat_i + 1, cat_name, count);
            } else {
                let _ = write!(buf, "\x1b[37m[{}] {} ({})\x1b[0m  ", cat_i + 1, cat_name, count);
            }
        }
        buf.push_str("\r\n\r\n");

        // Dynamic column width calculation that strictly fits without wrapping
        let col_sel = 5;
        let col_status = 8;
        let col_proj = (width / 6).clamp(10, 16);
        let col_cat = (width / 7).clamp(8, 14);
        let col_size = 9;
        let col_act = 8;

        let fixed_used = col_sel + col_status + col_proj + col_cat + col_size + col_act + 8;
        let col_path = if width > fixed_used + 10 {
            width - fixed_used - 2
        } else {
            12
        };

        // Table Top Border
        let _ = writeln!(
            buf,
            "\x1b[90m╭{}┬{}┬{}┬{}┬{}┬{}┬{}╮\x1b[0m\r",
            "─".repeat(col_sel),
            "─".repeat(col_status),
            "─".repeat(col_proj),
            "─".repeat(col_cat),
            "─".repeat(col_size),
            "─".repeat(col_act),
            "─".repeat(col_path)
        );

        // Table Header
        let _ = writeln!(
            buf,
            "\x1b[1;37m│ {:<col_sel$} │ {:<col_status$} │ {:<col_proj$} │ {:<col_cat$} │ {:>col_size$} │ {:<col_act$} │ {:<col_path$} │\x1b[0m\r",
            "Sel",
            "Verdict",
            "Project",
            "Category",
            "Size",
            "Active",
            "Artifact Path",
            col_sel = col_sel - 2,
            col_status = col_status - 2,
            col_proj = col_proj - 2,
            col_cat = col_cat - 2,
            col_size = col_size - 2,
            col_act = col_act - 2,
            col_path = col_path - 2
        );

        // Table Header Separator
        let _ = writeln!(
            buf,
            "\x1b[90m╞{}╪{}╪{}╪{}╪{}╪{}╪{}╡\x1b[0m\r",
            "═".repeat(col_sel),
            "═".repeat(col_status),
            "═".repeat(col_proj),
            "═".repeat(col_cat),
            "═".repeat(col_size),
            "═".repeat(col_act),
            "═".repeat(col_path)
        );

        // Viewport height calculation (leaves room for header, details card, and footer)
        let overhead_lines = 13;
        let visible_rows = if height > overhead_lines {
            height - overhead_lines
        } else {
            3
        };

        if cursor_idx >= scroll_offset + visible_rows {
            scroll_offset = cursor_idx.saturating_sub(visible_rows).saturating_add(1);
        } else if cursor_idx < scroll_offset {
            scroll_offset = cursor_idx;
        }

        let end_idx = (scroll_offset + visible_rows).min(filtered_indices.len());

        if filtered_indices.is_empty() {
            let empty_msg = "  No artifacts match this category filter.";
            let _ = writeln!(buf, "\x1b[90m│ {:<w$} │\x1b[0m\r", empty_msg, w = width.saturating_sub(4));
        } else {
            for (v_idx, &real_idx) in filtered_indices[scroll_offset..end_idx].iter().enumerate() {
                let display_row_idx = scroll_offset + v_idx;
                let is_highlighted = display_row_idx == cursor_idx;
                let item = &items[real_idx];

                let sel_marker = if item.selected {
                    "\x1b[32m[X]\x1b[0m"
                } else {
                    "\x1b[90m[ ]\x1b[0m"
                };

                let pointer = if is_highlighted { "\x1b[1;36m>\x1b[0m" } else { " " };

                let status_colored = if item.is_safe {
                    format!("\x1b[32m{:<width$}\x1b[0m", item.verdict_badge, width = col_status - 2)
                } else {
                    format!("\x1b[33m{:<width$}\x1b[0m", item.verdict_badge, width = col_status - 2)
                };

                let t_proj = truncate_to(&item.artifact.project_name, col_proj - 2);
                let t_cat = truncate_to(&item.artifact.category, col_cat - 2);
                let t_path = truncate_to(&item.artifact.path.display().to_string(), col_path - 2);

                if is_highlighted {
                    buf.push_str("\x1b[44m"); // Blue highlight for selected row
                }

                let _ = write!(
                    buf,
                    "│ {} {} │ {} │ {:<col_proj$} │ {:<col_cat$} │ {:>col_size$} │ {:<col_act$} │ {:<col_path$} │\x1b[0m\r\n",
                    pointer,
                    sel_marker,
                    status_colored,
                    t_proj,
                    t_cat,
                    item.size_str,
                    item.relative_time,
                    t_path,
                    col_proj = col_proj - 2,
                    col_cat = col_cat - 2,
                    col_size = col_size - 2,
                    col_act = col_act - 2,
                    col_path = col_path - 2
                );
            }
        }

        // Table Bottom Border
        let _ = writeln!(
            buf,
            "\x1b[90m╰{}┴{}┴{}┴{}┴{}┴{}┴{}╯\x1b[0m\r",
            "─".repeat(col_sel),
            "─".repeat(col_status),
            "─".repeat(col_proj),
            "─".repeat(col_cat),
            "─".repeat(col_size),
            "─".repeat(col_act),
            "─".repeat(col_path)
        );

        // 3. Highlighted Target Detail Card
        if let Some(&real_idx) = filtered_indices.get(cursor_idx) {
            let item = &items[real_idx];
            let card_inner = width.saturating_sub(6).max(30);
            let p_str = truncate_to(&item.artifact.path.display().to_string(), card_inner.saturating_sub(10));
            let v_str = truncate_to(&item.verdict_detail, card_inner.saturating_sub(10));
            let r_str = format!("Run `{}` to regenerate if ever needed", item.artifact.recreate_cmd);
            let r_trunc = truncate_to(&r_str, card_inner.saturating_sub(10));

            let _ = writeln!(buf, "\x1b[36m╭─ Target Details ─{}╮\x1b[0m\r", "─".repeat(width.saturating_sub(21)));
            let _ = writeln!(buf, "\x1b[36m│\x1b[0m Exact Path:  {:<w$} \x1b[36m│\x1b[0m\r", p_str, w = card_inner.saturating_sub(2));
            let _ = writeln!(buf, "\x1b[36m│\x1b[0m Assessment:  {:<w$} \x1b[36m│\x1b[0m\r", v_str, w = card_inner.saturating_sub(2));
            let _ = writeln!(buf, "\x1b[36m│\x1b[0m Restore Cmd: {:<w$} \x1b[36m│\x1b[0m\r", r_trunc, w = card_inner.saturating_sub(2));
            let _ = writeln!(buf, "\x1b[36m╰──────────────────{}╯\x1b[0m\r", "─".repeat(width.saturating_sub(21)));
        }

        // 4. Status Bar
        let selected_count = items.iter().filter(|i| i.selected).count();
        let selected_bytes: u64 = items
            .iter()
            .filter(|i| i.selected)
            .map(|i| i.artifact.size_bytes)
            .sum();

        if let Some(ref msg) = status_msg {
            let _ = writeln!(buf, "\x1b[1;32mNotice:\x1b[0m {}\r", msg);
        } else {
            let _ = writeln!(
                buf,
                "\x1b[33mSelected: {}/{} targets ({} reclaimable)  |  Category: {}\x1b[0m\r",
                selected_count,
                items.len(),
                format_bytes(selected_bytes),
                active_cat
            );
        }

        // 5. Controls
        buf.push_str("\x1b[90m[Up/Down/Click] Move  [Space] Select  [A] Select Safe  [Tab] Category  [O] Open  [Enter] Purge  [Q] Exit\x1b[0m");

        // Clear any remaining lines below the buffer to avoid screen remnants
        buf.push_str("\x1b[J");

        // Write entire frame in a single atomic syscall
        let mut out = stdout();
        let _ = out.write_all(buf.as_bytes());
        let _ = out.flush();

        // 6. Event Handling
        if let Ok(evt) = event::read() {
            match evt {
                Event::Key(key_event) => {
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
                Event::Mouse(mouse_event) => {
                    match mouse_event.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            let click_row = mouse_event.row as usize;
                            // Table data rows start at line 6
                            let table_start_row = 6;
                            if click_row >= table_start_row {
                                let relative_click = click_row - table_start_row;
                                if relative_click < (end_idx - scroll_offset) {
                                    let target_cursor = scroll_offset + relative_click;
                                    if target_cursor < filtered_indices.len() {
                                        cursor_idx = target_cursor;
                                        // If clicked near the checkbox on the left, toggle selection
                                        if mouse_event.column <= 8 {
                                            if let Some(&real_idx) = filtered_indices.get(cursor_idx) {
                                                if let Some(item) = items.get_mut(real_idx) {
                                                    item.selected = !item.selected;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if !filtered_indices.is_empty() && cursor_idx + 1 < filtered_indices.len() {
                                cursor_idx += 1;
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            cursor_idx = cursor_idx.saturating_sub(1);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

fn truncate_to(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max_len {
        let keep = max_len.saturating_sub(2);
        let prefix: String = s.chars().take(keep).collect();
        format!("{}..", prefix)
    } else {
        s.to_string()
    }
}
