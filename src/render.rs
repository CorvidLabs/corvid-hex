use crate::app::{App, Mode};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

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
            Constraint::Min(1),   // Hex view
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_hex_view(f, app, chunks[1]);
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

fn draw_hex_view(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::NONE);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = inner.height as usize;
    let bpr = app.bytes_per_row;

    // Build search hit set for quick lookup
    let search_hits: std::collections::HashSet<usize> = app.search_results.iter().copied().collect();
    // Current match offset (for highlighting the active match differently)
    let current_match: Option<usize> = if !app.search_results.is_empty() {
        Some(app.search_results[app.search_index])
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
                let is_current_match = current_match == Some(offset);
                let is_selected = selection.is_some_and(|(lo, hi)| offset >= lo && offset <= hi);

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
                } else {
                    Style::default().fg(byte_color(byte, modified))
                };

                hex_spans.push(Span::styled(hex, style));
                hex_spans.push(Span::raw(if col == 7 { "  " } else { " " }));
            } else {
                hex_spans.push(Span::raw("   "));
            }
        }

        f.render_widget(
            Paragraph::new(Line::from(hex_spans)),
            Rect::new(inner.x + 9, y, (bpr * 3 + 2) as u16, 1),
        );

        // ASCII column
        let ascii_x = inner.x + 9 + (bpr * 3 + 2) as u16 + 1;
        let mut ascii_spans: Vec<Span> = Vec::with_capacity(bpr + 2);
        ascii_spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));

        for col in 0..bpr {
            let offset = row_offset + col;
            if let Some(byte) = app.buffer.get(offset) {
                let modified = app.buffer.is_modified(offset);
                let is_cursor = offset == app.cursor;
                let is_search = search_hits.contains(&offset);
                let is_current_match = current_match == Some(offset);
                let is_selected = selection.is_some_and(|(lo, hi)| offset >= lo && offset <= hi);
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
                } else {
                    Style::default().fg(byte_color(byte, modified))
                };

                ascii_spans.push(Span::styled(ch, style));
            } else {
                ascii_spans.push(Span::raw(" "));
            }
        }

        ascii_spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));

        f.render_widget(
            Paragraph::new(Line::from(ascii_spans)),
            Rect::new(ascii_x, y, (bpr + 2) as u16, 1),
        );
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode_style = match app.mode {
        Mode::Normal => Style::default().fg(Color::Black).bg(Color::Blue),
        Mode::Visual => Style::default().fg(Color::Black).bg(Color::Yellow),
        Mode::EditHex | Mode::EditAscii => Style::default().fg(Color::Black).bg(Color::Green),
        Mode::Command | Mode::Search => Style::default().fg(Color::Black).bg(Color::Magenta),
    };

    let mode_label = format!(" {} ", app.mode.label());

    let input_part = match app.mode {
        Mode::Command => format!(":{}", app.command_input),
        Mode::Search => format!("/{}", app.search_input),
        _ => app
            .status_message
            .clone()
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
