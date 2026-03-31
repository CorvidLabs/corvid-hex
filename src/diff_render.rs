use crate::diff::DiffState;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

const COLOR_DIFF: Color = Color::Red;
const COLOR_DIFF_BG: Color = Color::Rgb(60, 0, 0);
const COLOR_CURSOR: Color = Color::Black;
const COLOR_CURSOR_BG: Color = Color::White;
const COLOR_NULL: Color = Color::DarkGray;
const COLOR_PRINTABLE: Color = Color::Cyan;
const COLOR_HIGH: Color = Color::Yellow;
const COLOR_ADDITION: Color = Color::Green;

fn byte_color(b: u8) -> Color {
    if b == 0 {
        COLOR_NULL
    } else if b.is_ascii_graphic() || b == b' ' {
        COLOR_PRINTABLE
    } else {
        COLOR_HIGH
    }
}

pub fn draw_diff(f: &mut Frame, state: &mut DiffState) {
    let area = f.area();

    state.visible_rows = area.height.saturating_sub(4) as usize;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(1), // Stats
            Constraint::Min(1),   // Diff view
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    draw_header(f, state, chunks[0]);
    draw_stats_bar(f, state, chunks[1]);
    draw_split_view(f, state, chunks[2]);
    draw_status_bar(f, state, chunks[3]);
}

fn draw_header(f: &mut Frame, state: &DiffState, area: Rect) {
    let text = format!(
        " DIFF: {} ({} bytes) vs {} ({} bytes)",
        state.left_name,
        state.left_data.len(),
        state.right_name,
        state.right_data.len(),
    );
    let header = Paragraph::new(text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(header, area);
}

fn draw_stats_bar(f: &mut Frame, state: &DiffState, area: Rect) {
    let stats = state.stats();
    let xor_indicator = if state.xor_view { " [XOR]" } else { "" };
    let text = format!(
        " {} differences, {:.1}% match{}{}",
        stats.diff_count,
        stats.match_percentage,
        match stats.first_diff {
            Some(offset) => format!(", first diff at 0x{:08X}", offset),
            None => String::new(),
        },
        xor_indicator,
    );
    let bar = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow).bg(Color::Rgb(30, 30, 30)));
    f.render_widget(bar, area);
}

fn draw_split_view(f: &mut Frame, state: &mut DiffState, area: Rect) {
    let half_width = area.width / 2;

    // Auto-fit bytes_per_row for each half panel.
    // Layout per panel: 9 (offset) + 1 (space) + bpr*3 (hex) = 10 + bpr*3
    // We skip the ASCII pane to fit more in the split view.
    let max_bpr = (half_width as usize).saturating_sub(10) / 3;
    let bpr = state.bytes_per_row.min(max_bpr).max(1);
    state.bytes_per_row = bpr;

    let rows = area.height as usize;

    // Draw left panel
    draw_panel(
        f,
        state,
        Rect::new(area.x, area.y, half_width, area.height),
        PanelSide::Left,
        bpr,
        rows,
    );

    // Draw separator
    for row in 0..rows {
        let y = area.y + row as u16;
        let sep_x = area.x + half_width;
        if sep_x < area.x + area.width {
            f.render_widget(
                Paragraph::new("│").style(Style::default().fg(Color::DarkGray)),
                Rect::new(sep_x, y, 1, 1),
            );
        }
    }

    // Draw right panel
    let right_x = area.x + half_width + 1;
    let right_width = area.width.saturating_sub(half_width + 1);
    if right_width > 0 {
        draw_panel(
            f,
            state,
            Rect::new(right_x, area.y, right_width, area.height),
            PanelSide::Right,
            bpr,
            rows,
        );
    }
}

enum PanelSide {
    Left,
    Right,
}

fn draw_panel(
    f: &mut Frame,
    state: &DiffState,
    area: Rect,
    side: PanelSide,
    bpr: usize,
    rows: usize,
) {
    for row_idx in 0..rows {
        let data_row = state.scroll_offset + row_idx;
        let row_offset = data_row * bpr;

        if row_offset >= state.max_len() {
            break;
        }

        let y = area.y + row_idx as u16;

        // Offset column
        let offset_str = format!("{:08X}", row_offset);
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                offset_str,
                Style::default().fg(Color::DarkGray),
            ))),
            Rect::new(area.x, y, 9.min(area.width), 1),
        );

        // Hex bytes
        let mut hex_spans: Vec<Span> = Vec::with_capacity(bpr * 3 + 1);
        hex_spans.push(Span::raw(" "));

        for col in 0..bpr {
            let offset = row_offset + col;
            if offset >= state.max_len() {
                break;
            }

            let (byte_opt, other_opt) = match side {
                PanelSide::Left => (state.left_byte(offset), state.right_byte(offset)),
                PanelSide::Right => {
                    if state.xor_view {
                        // XOR view: show XOR of left and right bytes
                        let l = state.left_byte(offset);
                        let r = state.right_byte(offset);
                        let xor = match (l, r) {
                            (Some(a), Some(b)) => Some(a ^ b),
                            (Some(_), None) | (None, Some(_)) => Some(0xFF),
                            (None, None) => None,
                        };
                        (xor, state.left_byte(offset))
                    } else {
                        (state.right_byte(offset), state.left_byte(offset))
                    }
                }
            };

            let is_cursor = offset == state.cursor;
            let is_diff = state.is_diff(offset);
            let is_addition = byte_opt.is_some() && other_opt.is_none();

            let hex = match byte_opt {
                Some(b) => format!("{:02X}", b),
                None => "  ".to_string(),
            };

            let style = if is_cursor {
                Style::default().fg(COLOR_CURSOR).bg(COLOR_CURSOR_BG)
            } else if is_addition {
                Style::default()
                    .fg(COLOR_ADDITION)
                    .add_modifier(Modifier::BOLD)
            } else if is_diff {
                Style::default().fg(COLOR_DIFF).bg(COLOR_DIFF_BG)
            } else {
                match byte_opt {
                    Some(b) => Style::default().fg(byte_color(b)),
                    None => Style::default().fg(COLOR_NULL),
                }
            };

            hex_spans.push(Span::styled(hex, style));
            hex_spans.push(Span::raw(" "));
        }

        let hex_x = area.x + 9;
        let hex_w = ((bpr * 3 + 1) as u16).min(area.width.saturating_sub(9));
        if hex_w > 0 {
            f.render_widget(
                Paragraph::new(Line::from(hex_spans)),
                Rect::new(hex_x, y, hex_w, 1),
            );
        }
    }
}

fn draw_status_bar(f: &mut Frame, state: &DiffState, area: Rect) {
    let mode_label = if state.xor_view { " DIFF-XOR " } else { " DIFF " };
    let mode_style = Style::default().fg(Color::Black).bg(Color::Magenta);

    let message = state.status_message.clone().unwrap_or_default();
    let right_info = format!("0x{:08X} ({}) ", state.cursor, state.cursor);

    let available = area.width as usize;
    let mode_len = mode_label.len();
    let right_len = right_info.len();
    let mid_len = available.saturating_sub(mode_len + right_len);

    let padded_msg = format!(" {:<width$}", message, width = mid_len.saturating_sub(1));

    let line = Line::from(vec![
        Span::styled(mode_label, mode_style),
        Span::styled(
            padded_msg,
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ),
        Span::styled(
            right_info,
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ),
    ]);

    f.render_widget(Paragraph::new(line), area);
}
