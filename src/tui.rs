use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{stdout, IsTerminal, Write};

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
    relative_time: String,
}

pub fn run_interactive_navigator(artifacts: &[ArtifactInfo]) -> Option<Vec<ArtifactInfo>> {
    if artifacts.is_empty() {
        return None;
    }

    if !std::io::stdin().is_terminal() || !stdout().is_terminal() {
        return None;
    }

    let _guard = RawModeGuard::enter().ok()?;

    let mut items: Vec<TuiItem> = artifacts
        .iter()
        .map(|art| {
            let analysis = analyze_folder_integrity(art);
            let is_safe = !analysis.is_needed;
            let relative_time = format_time_relative(art.modified_at);
            TuiItem {
                artifact: art.clone(),
                selected: is_safe,
                is_safe,
                relative_time,
            }
        })
        .collect();

    let mut cursor_idx: usize = 0;
    let mut scroll_offset: usize = 0;
    let mut status_msg: Option<String> = None;

    loop {
        let (term_width, term_height) = terminal::size().unwrap_or((80, 24));
        let width = term_width as usize;
        let height = term_height as usize;

        // Render frame
        let mut out = stdout();
        let _ = execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All));

        // Header
        let _ = execute!(
            out,
            SetForegroundColor(Color::Cyan),
            Print("================================================================================\r\n"),
            Print("                     >> BloatHound Interactive Navigator <<                     \r\n"),
            Print("================================================================================\r\n"),
            ResetColor,
            SetForegroundColor(Color::DarkGrey),
            Print(" Controls: [Up/Down] Navigate | [Space] Toggle | [A] Select All | [O] Open | [Enter] Purge | [Q] Exit\r\n\r\n"),
            ResetColor,
        );

        let header_lines = 5;
        let footer_lines = 3;
        let visible_rows = if height > (header_lines + footer_lines) {
            height - header_lines - footer_lines
        } else {
            5
        };

        // Adjust scroll offset
        if cursor_idx >= scroll_offset + visible_rows {
            scroll_offset = cursor_idx.saturating_sub(visible_rows).saturating_add(1);
        } else if cursor_idx < scroll_offset {
            scroll_offset = cursor_idx;
        }

        let end_idx = (scroll_offset + visible_rows).min(items.len());

        for (idx, item) in items[scroll_offset..end_idx].iter().enumerate() {
            let absolute_idx = scroll_offset + idx;
            let is_highlighted = absolute_idx == cursor_idx;

            if is_highlighted {
                let _ = execute!(out, SetBackgroundColor(Color::DarkBlue), SetForegroundColor(Color::White));
            }

            let pointer = if is_highlighted { "> " } else { "  " };
            let checkbox = if item.selected { "[X]" } else { "[ ]" };

            let verdict_str = if item.is_safe {
                "[SAFE TO DELETE]"
            } else {
                "[MATTERS - KEEP]"
            };

            let size_str = format_bytes(item.artifact.size_bytes);
            let display_name = format!("{}/{}", item.artifact.project_name, item.artifact.pattern);

            let row_line = format!(
                "{}{} {:<22} {:>9}  {:<16}  {}",
                pointer,
                checkbox,
                display_name,
                size_str,
                verdict_str,
                item.relative_time
            );

            let truncated = if row_line.len() > width && width > 3 {
                &row_line[..width - 3]
            } else {
                &row_line
            };

            let _ = execute!(out, Print(truncated), ResetColor, Print("\r\n"));
        }

        // Footer / Summary
        let selected_count = items.iter().filter(|i| i.selected).count();
        let selected_bytes: u64 = items
            .iter()
            .filter(|i| i.selected)
            .map(|i| i.artifact.size_bytes)
            .sum();

        let _ = execute!(out, cursor::MoveTo(0, (height.saturating_sub(2)) as u16));

        if let Some(ref msg) = status_msg {
            let _ = execute!(out, SetForegroundColor(Color::Green), Print(msg), ResetColor, Print("\r\n"));
        } else {
            let summary = format!(
                "Selected: {}/{} targets ({} to reclaim)",
                selected_count,
                items.len(),
                format_bytes(selected_bytes)
            );
            let _ = execute!(out, SetForegroundColor(Color::Yellow), Print(summary), ResetColor, Print("\r\n"));
        }

        let _ = out.flush();

        // Handle user input
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
                    if cursor_idx + 1 < items.len() {
                        cursor_idx += 1;
                    }
                }
                KeyCode::Home => {
                    cursor_idx = 0;
                }
                KeyCode::End => {
                    if !items.is_empty() {
                        cursor_idx = items.len() - 1;
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(item) = items.get_mut(cursor_idx) {
                        item.selected = !item.selected;
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    let any_unselected = items.iter().any(|i| !i.selected);
                    for item in &mut items {
                        item.selected = any_unselected;
                    }
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    if let Some(item) = items.get(cursor_idx) {
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
