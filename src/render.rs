use crate::app::{App, Mode};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

const COLOR_NULL: Color = Color::DarkGray;
const COLOR_PRINTABLE: Color = Color::Cyan;
const COLOR_HIGH: Color = Color::Yellow;
const COLOR_MODIFIED: Color = Color::Red;
const COLOR_CURSOR: Color = Color::Black;
const COLOR_CURSOR_BG: Color = Color::White;
const COLOR_SEARCH_HIT: Color = Color::Black;
const COLOR_SEARCH_BG: Color = Color::Yellow;
const COLOR_CURRENT_MATCH_BG: Color = Color::Magenta;
const COLOR_SELECTION_BG: Color = Color::Blue;

/// Cycling color palette for template field highlighting.
/// Each (fg, bg) pair is used for fields by index modulo palette length.
const TEMPLATE_PALETTE: &[(Color, Color)] = &[
    (Color::Black, Color::LightGreen),
    (Color::Black, Color::LightBlue),
    (Color::Black, Color::LightMagenta),
    (Color::Black, Color::LightCyan),
    (Color::Black, Color::LightYellow),
    (Color::White, Color::Green),
    (Color::White, Color::Blue),
    (Color::White, Color::Magenta),
];

fn byte_color(b: u8, modified: bool) -> Color {
    if modified {
        COLOR_MODIFIED
    } else if b == 0 {
        COLOR_NULL
    } else if b.is_ascii_graphic() || b == b' ' {
        COLOR_PRINTABLE
    } else {
        COLOR_HIGH
    }
}

fn ascii_char(b: u8) -> char {
    if b.is_ascii_graphic() || b == b' ' {
        b as char
    } else {
        '.'
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Update visible rows based on terminal height (minus 2 for header + status)
    app.visible_rows = area.height.saturating_sub(3) as usize;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(1),    // Hex view (+ optional strings panel)
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    draw_header(f, app, chunks[0]);

    // When the strings panel is visible, split the main area horizontally
    if app.strings_panel.visible {
        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);
        draw_hex_view(f, app, panels[0]);
        draw_strings_panel(f, app, panels[1]);
    } else {
        draw_hex_view(f, app, chunks[1]);
    }

    draw_status_bar(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let filename = app.buffer.path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "???".to_string());
    let dirty = if app.buffer.is_dirty() { " [+]" } else { "" };
    let size = app.buffer.len();

    let text = format!(" {filename}{dirty} — {size} bytes (0x{size:X})");
    let header = Paragraph::new(text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(header, area);
}

fn draw_hex_view(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default().borders(Borders::NONE);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Cache the hex view area for mouse hit-testing
    app.hex_view_area = inner;

    let rows = inner.height as usize;

    // Auto-fit bytes_per_row to terminal width.
    // Layout: 9 (offset) + (bpr*3 + 2) (hex) + 1 (gap) + (bpr + 2) (ascii) = 14 + 4*bpr
    let max_bpr = (inner.width as usize).saturating_sub(14) / 4;
    let bpr = app.requested_bytes_per_row.min(max_bpr).max(1);
    app.bytes_per_row = bpr;

    // Build search hit set for quick lookup — include all bytes in each match span
    let pattern_len = app.search_pattern_len.max(1);
    let search_hits: std::collections::HashSet<usize> = app.search_results.iter()
        .flat_map(|&start| start..start + pattern_len)
        .collect();
    // Current match range (for highlighting the active match differently)
    let current_match_range: Option<(usize, usize)> = if !app.search_results.is_empty() {
        let start = app.search_results[app.search_index];
        Some((start, start + pattern_len))
    } else {
        None
    };
    // Selection range
    let selection = app.selection_range();

    for row_idx in 0..rows {
        let data_row = app.scroll_offset + row_idx;
        let row_offset = data_row * bpr;

        if row_offset >= app.buffer.len() {
            break;
        }

        let y = inner.y + row_idx as u16;

        // Offset column
        let offset_str = format!("{:08X}", row_offset);
        let offset_span = Span::styled(
            offset_str,
            Style::default().fg(Color::DarkGray),
        );
        f.render_widget(
            Paragraph::new(Line::from(offset_span)),
            Rect::new(inner.x, y, 9, 1),
        );

        // Hex bytes column
        let mut hex_spans: Vec<Span> = Vec::with_capacity(bpr * 3 + 1);
        hex_spans.push(Span::raw(" "));

        for col in 0..bpr {
            let offset = row_offset + col;
            if let Some(byte) = app.buffer.get(offset) {
                let modified = app.buffer.is_modified(offset);
                let is_cursor = offset == app.cursor;
                let is_search = search_hits.contains(&offset);
                let is_current_match = current_match_range.is_some_and(|(lo, hi)| offset >= lo && offset < hi);
                let is_selected = selection.is_some_and(|(lo, hi)| offset >= lo && offset <= hi);
                let template_field = if app.show_template_overlay {
                    app.template_field_map.get(&offset)
                } else {
                    None
                };

                let hex = format!("{:02X}", byte);

                let style = if is_cursor {
                    match app.mode {
                        Mode::EditHex => Style::default()
                            .fg(COLOR_CURSOR)
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                        _ => Style::default()
                            .fg(COLOR_CURSOR)
                            .bg(COLOR_CURSOR_BG),
                    }
                } else if is_selected {
                    Style::default().fg(Color::White).bg(COLOR_SELECTION_BG)
                } else if is_current_match {
                    Style::default().fg(COLOR_SEARCH_HIT).bg(COLOR_CURRENT_MATCH_BG).add_modifier(Modifier::BOLD)
                } else if is_search {
                    Style::default().fg(COLOR_SEARCH_HIT).bg(COLOR_SEARCH_BG)
                } else if let Some((_, field_idx)) = template_field {
                    let (fg, bg) = TEMPLATE_PALETTE[field_idx % TEMPLATE_PALETTE.len()];
                    Style::default().fg(fg).bg(bg)
                } else {
                    Style::default().fg(byte_color(byte, modified))
                };

                hex_spans.push(Span::styled(hex, style));
                hex_spans.push(Span::raw(if col == 7 { "  " } else { " " }));
            } else {
                hex_spans.push(Span::raw("   "));
            }
        }

        let hex_x = inner.x + 9;
        let hex_w = ((bpr * 3 + 2) as u16).min(inner.width.saturating_sub(9));
        f.render_widget(
            Paragraph::new(Line::from(hex_spans)),
            Rect::new(hex_x, y, hex_w, 1),
        );

        // ASCII column
        let ascii_x = hex_x + hex_w + 1;
        let ascii_w = ((bpr + 2) as u16).min((inner.x + inner.width).saturating_sub(ascii_x));
        let mut ascii_spans: Vec<Span> = Vec::with_capacity(bpr + 2);
        ascii_spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));

        for col in 0..bpr {
            let offset = row_offset + col;
            if let Some(byte) = app.buffer.get(offset) {
                let modified = app.buffer.is_modified(offset);
                let is_cursor = offset == app.cursor;
                let is_search = search_hits.contains(&offset);
                let is_current_match = current_match_range.is_some_and(|(lo, hi)| offset >= lo && offset < hi);
                let is_selected = selection.is_some_and(|(lo, hi)| offset >= lo && offset <= hi);
                let template_field = if app.show_template_overlay {
                    app.template_field_map.get(&offset)
                } else {
                    None
                };
                let ch = ascii_char(byte).to_string();

                let style = if is_cursor {
                    match app.mode {
                        Mode::EditAscii => Style::default()
                            .fg(COLOR_CURSOR)
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                        _ => Style::default()
                            .fg(COLOR_CURSOR)
                            .bg(COLOR_CURSOR_BG),
                    }
                } else if is_selected {
                    Style::default().fg(Color::White).bg(COLOR_SELECTION_BG)
                } else if is_current_match {
                    Style::default().fg(COLOR_SEARCH_HIT).bg(COLOR_CURRENT_MATCH_BG).add_modifier(Modifier::BOLD)
                } else if is_search {
                    Style::default().fg(COLOR_SEARCH_HIT).bg(COLOR_SEARCH_BG)
                } else if let Some((_, field_idx)) = template_field {
                    let (fg, bg) = TEMPLATE_PALETTE[field_idx % TEMPLATE_PALETTE.len()];
                    Style::default().fg(fg).bg(bg)
                } else {
                    Style::default().fg(byte_color(byte, modified))
                };

                ascii_spans.push(Span::styled(ch, style));
            } else {
                ascii_spans.push(Span::raw(" "));
            }
        }

        ascii_spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));

        if ascii_w > 0 {
            f.render_widget(
                Paragraph::new(Line::from(ascii_spans)),
                Rect::new(ascii_x, y, ascii_w, 1),
            );
        }
    }
}

fn draw_strings_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let panel = &app.strings_panel;
    let count = panel.results.len();
    let title = format!(" Strings ({count}, min:{}) ", panel.min_length);

    let border_style = if app.mode == Mode::Strings {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Cache visible rows for scroll management in input handler
    app.strings_panel.visible_rows = inner.height as usize;

    let visible_rows = inner.height as usize;
    let scroll = app.strings_panel.scroll;
    let selected = app.strings_panel.selected;

    for i in 0..visible_rows {
        let idx = scroll + i;
        if idx >= count {
            break;
        }
        let entry = &app.strings_panel.results[idx];
        let is_selected = idx == selected;

        // Build line: "  0xOFFSET  KIND  text..."
        let prefix = format!(" {:08X}  {:8}  ", entry.offset, entry.kind.label());
        let text_space = (inner.width as usize).saturating_sub(prefix.len());
        let truncated: String = entry.text.chars().take(text_space).collect();
        let line_text = format!("{prefix}{truncated}");

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            let kind_color = match entry.kind {
                crate::strings::StringKind::Ascii => Color::Green,
                crate::strings::StringKind::Utf8 => Color::Cyan,
                crate::strings::StringKind::Utf16Le => Color::Yellow,
                crate::strings::StringKind::Utf16Be => Color::Magenta,
            };
            Style::default().fg(kind_color)
        };

        let y = inner.y + i as u16;
        f.render_widget(
            Paragraph::new(line_text).style(style),
            Rect::new(inner.x, y, inner.width, 1),
        );
    }

    // Scrollbar on the right edge (only when content overflows)
    if count > visible_rows && inner.width > 1 {
        let mut scrollbar_state = ScrollbarState::new(count.saturating_sub(visible_rows))
            .position(scroll);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        f.render_stateful_widget(
            scrollbar,
            Rect::new(area.x + area.width - 1, area.y + 1, 1, area.height.saturating_sub(2)),
            &mut scrollbar_state,
        );
    }

    // Show hint at the bottom when panel is focused
    if app.mode == Mode::Strings && inner.height > 0 {
        let hint = " Enter:jump  x:export  Esc:close ";
        let hint_len = hint.len().min(inner.width as usize) as u16;
        let hint_x = inner.x + inner.width.saturating_sub(hint_len);
        let hint_y = area.y + area.height.saturating_sub(1);
        f.render_widget(
            Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
            Rect::new(hint_x, hint_y, hint_len, 1),
        );
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode_style = match app.mode {
        Mode::Normal => Style::default().fg(Color::Black).bg(Color::Blue),
        Mode::Visual => Style::default().fg(Color::Black).bg(Color::Yellow),
        Mode::EditHex | Mode::EditAscii => Style::default().fg(Color::Black).bg(Color::Green),
        Mode::Command | Mode::Search => Style::default().fg(Color::Black).bg(Color::Magenta),
        Mode::Strings => Style::default().fg(Color::Black).bg(Color::Cyan),
    };

    let mode_label = format!(" {} ", app.mode.label());

    let input_part = match app.mode {
        Mode::Command => format!(":{}", app.command_input),
        Mode::Search => format!("/{}", app.search_input),
        _ => app
            .template_field_info_at_cursor()
            .or_else(|| app.status_message.clone())
            .unwrap_or_default(),
    };

    let right_info = format!(
        "0x{:08X} ({}) ",
        app.cursor, app.cursor
    );

    let available = area.width as usize;
    let mode_len = mode_label.len();
    let right_len = right_info.len();
    let mid_len = available.saturating_sub(mode_len + right_len);

    let padded_input = format!(" {:<width$}", input_part, width = mid_len.saturating_sub(1));

    let line = Line::from(vec![
        Span::styled(mode_label, mode_style),
        Span::styled(padded_input, Style::default().fg(Color::White).bg(Color::DarkGray)),
        Span::styled(right_info, Style::default().fg(Color::White).bg(Color::DarkGray)),
    ]);

    f.render_widget(Paragraph::new(line), area);
}
