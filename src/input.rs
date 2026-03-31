use crate::app::{App, Mode};
use crate::search;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

/// Handle a key event. Returns true if the app should quit.
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match app.mode {
        Mode::Normal => handle_normal(app, key),
        Mode::Visual => handle_visual(app, key),
        Mode::EditHex => handle_edit_hex(app, key),
        Mode::EditAscii => handle_edit_ascii(app, key),
        Mode::Command => handle_command(app, key),
        Mode::Search => handle_search(app, key),
        Mode::Strings => handle_strings(app, key),
    }
}

/// Handle a mouse event.
pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Check if the click lands on the entropy panel.
            if app.show_entropy {
                let ep = app.entropy_panel_area;
                if ep.width > 0
                    && mouse.column >= ep.x
                    && mouse.column < ep.x + ep.width
                    && mouse.row >= ep.y
                    && mouse.row < ep.y + ep.height
                {
                    let panel_height = ep.height as usize;
                    let file_len = app.buffer.len();
                    if panel_height > 0 && file_len > 0 {
                        let row = (mouse.row - ep.y) as usize;
                        let seg_start = (row * file_len) / panel_height;
                        app.move_cursor_to(seg_start);
                        app.mode = Mode::Normal;
                    }
                    return;
                }
            }

            if let Some(offset) = app.offset_from_screen(mouse.column, mouse.row) {
                // Exit any text-input modes on click
                match app.mode {
                    Mode::Command | Mode::Search => {
                        app.mode = Mode::Normal;
                    }
                    Mode::EditHex => {
                        app.hex_nibble = None;
                    }
                    _ => {}
                }
                // If already in visual mode, restart selection; otherwise just position cursor
                if app.mode == Mode::Visual {
                    app.selection_anchor = Some(offset);
                }
                app.move_cursor_to(offset);
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if let Some(offset) = app.offset_from_screen(mouse.column, mouse.row) {
                // Start visual selection on drag if not already in visual mode
                if app.mode != Mode::Visual {
                    app.mode = Mode::Visual;
                    app.selection_anchor = Some(app.cursor);
                }
                app.move_cursor_to(offset);
            }
        }
        MouseEventKind::ScrollDown => {
            let scroll_lines = 3;
            app.move_cursor((scroll_lines * app.bytes_per_row) as isize);
        }
        MouseEventKind::ScrollUp => {
            let scroll_lines = 3;
            app.move_cursor(-((scroll_lines * app.bytes_per_row) as isize));
        }
        _ => {}
    }
}

fn handle_normal(app: &mut App, key: KeyEvent) -> bool {
    app.status_message = None;

    // Handle pending bookmark operations (m + letter, ' + letter)
    if let Some(pending) = app.pending_bookmark.take() {
        if let KeyCode::Char(c) = key.code {
            if c.is_ascii_lowercase() {
                match pending {
                    'm' => {
                        app.bookmarks.insert(c, app.cursor);
                        app.status_message = Some(format!("Bookmark '{c}' set at 0x{:X}", app.cursor));
                    }
                    '\'' => {
                        if let Some(&offset) = app.bookmarks.get(&c) {
                            app.move_cursor_to(offset);
                            app.status_message = Some(format!("Jumped to bookmark '{c}'"));
                        } else {
                            app.status_message = Some(format!("Bookmark '{c}' not set"));
                        }
                    }
                    _ => {}
                }
                return false;
            }
        }
        // Invalid key after m/' — cancel silently
        app.status_message = Some("Bookmark cancelled".to_string());
        return false;
    }

    match key.code {
        // Mode switches
        KeyCode::Char(':') => {
            app.mode = Mode::Command;
            app.command_input.clear();
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Search;
            app.search_input.clear();
        }
        KeyCode::Char('i') => {
            app.mode = Mode::EditHex;
            app.hex_nibble = None;
        }
        KeyCode::Char('a') => {
            app.mode = Mode::EditAscii;
        }

        // Navigation
        KeyCode::Char('h') | KeyCode::Left => app.move_cursor(-1),
        KeyCode::Char('l') | KeyCode::Right => app.move_cursor(1),
        KeyCode::Char('k') | KeyCode::Up => app.move_cursor(-(app.bytes_per_row as isize)),
        KeyCode::Char('j') | KeyCode::Down => app.move_cursor(app.bytes_per_row as isize),

        KeyCode::Home | KeyCode::Char('0') => {
            let row_start = (app.cursor / app.bytes_per_row) * app.bytes_per_row;
            app.move_cursor_to(row_start);
        }
        KeyCode::End | KeyCode::Char('$') => {
            let row_end = ((app.cursor / app.bytes_per_row) + 1) * app.bytes_per_row - 1;
            app.move_cursor_to(row_end);
        }
        KeyCode::Char('g') => {
            app.move_cursor_to(0);
        }
        KeyCode::Char('G') => {
            if !app.buffer.is_empty() {
                app.move_cursor_to(app.buffer.len() - 1);
            }
        }

        KeyCode::PageDown => app.page_down(),
        KeyCode::PageUp => app.page_up(),

        // Ctrl-D / Ctrl-U for half-page
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let half = (app.visible_rows / 2) * app.bytes_per_row;
            app.move_cursor(half as isize);
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let half = (app.visible_rows / 2) * app.bytes_per_row;
            app.move_cursor(-(half as isize));
        }

        // Undo/Redo
        KeyCode::Char('u') => {
            if let Some(offset) = app.buffer.undo() {
                app.move_cursor_to(offset);
                app.status_message = Some("Undo".to_string());
            } else {
                app.status_message = Some("Nothing to undo".to_string());
            }
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(offset) = app.buffer.redo() {
                app.move_cursor_to(offset);
                app.status_message = Some("Redo".to_string());
            } else {
                app.status_message = Some("Nothing to redo".to_string());
            }
        }

        // Visual mode
        KeyCode::Char('v') => {
            app.mode = Mode::Visual;
            app.selection_anchor = Some(app.cursor);
        }

        // Paste
        KeyCode::Char('p') => {
            if app.clipboard.is_empty() {
                app.status_message = Some("Nothing to paste".to_string());
            } else {
                let count = app.paste();
                app.status_message = Some(format!("Pasted {count} bytes"));
            }
        }

        // Search navigation
        KeyCode::Char('n') => search::next_search_result(app),
        KeyCode::Char('N') => search::prev_search_result(app),

        // Bookmarks
        KeyCode::Char('m') => {
            app.pending_bookmark = Some('m');
        }
        KeyCode::Char('\'') => {
            app.pending_bookmark = Some('\'');
        }

        _ => {}
    }
    false
}

fn handle_visual(app: &mut App, key: KeyEvent) -> bool {
    app.status_message = None;

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.selection_anchor = None;
        }

        // Navigation extends selection
        KeyCode::Char('h') | KeyCode::Left => app.move_cursor(-1),
        KeyCode::Char('l') | KeyCode::Right => app.move_cursor(1),
        KeyCode::Char('k') | KeyCode::Up => app.move_cursor(-(app.bytes_per_row as isize)),
        KeyCode::Char('j') | KeyCode::Down => app.move_cursor(app.bytes_per_row as isize),

        KeyCode::Home | KeyCode::Char('0') => {
            let row_start = (app.cursor / app.bytes_per_row) * app.bytes_per_row;
            app.move_cursor_to(row_start);
        }
        KeyCode::End | KeyCode::Char('$') => {
            let row_end = ((app.cursor / app.bytes_per_row) + 1) * app.bytes_per_row - 1;
            app.move_cursor_to(row_end);
        }
        KeyCode::Char('g') => app.move_cursor_to(0),
        KeyCode::Char('G') => {
            if !app.buffer.is_empty() {
                app.move_cursor_to(app.buffer.len() - 1);
            }
        }

        // Yank
        KeyCode::Char('y') => {
            let count = app.yank_selection();
            app.mode = Mode::Normal;
            app.status_message = Some(format!("Yanked {count} bytes"));
        }

        _ => {}
    }
    false
}

fn handle_edit_hex(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.hex_nibble = None;
        }
        KeyCode::Char(c) if c.is_ascii_hexdigit() => {
            let Some(nibble) = c.to_digit(16).map(|d| d as u8) else { return false; };
            if let Some(high) = app.hex_nibble {
                // Second nibble — write the byte
                let value = (high << 4) | nibble;
                app.buffer.set(app.cursor, value);
                app.hex_nibble = None;
                app.move_cursor(1);
            } else {
                // First nibble — store and wait
                app.hex_nibble = Some(nibble);
            }
        }
        // Navigation in edit mode
        KeyCode::Left => {
            app.hex_nibble = None;
            app.move_cursor(-1);
        }
        KeyCode::Right => {
            app.hex_nibble = None;
            app.move_cursor(1);
        }
        KeyCode::Up => {
            app.hex_nibble = None;
            app.move_cursor(-(app.bytes_per_row as isize));
        }
        KeyCode::Down => {
            app.hex_nibble = None;
            app.move_cursor(app.bytes_per_row as isize);
        }
        KeyCode::Tab => {
            app.hex_nibble = None;
            app.mode = Mode::EditAscii;
        }
        _ => {}
    }
    false
}

fn handle_edit_ascii(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Char(c) if c.is_ascii() && !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.buffer.set(app.cursor, c as u8);
            app.move_cursor(1);
        }
        KeyCode::Left => app.move_cursor(-1),
        KeyCode::Right => app.move_cursor(1),
        KeyCode::Up => app.move_cursor(-(app.bytes_per_row as isize)),
        KeyCode::Down => app.move_cursor(app.bytes_per_row as isize),
        KeyCode::Tab => {
            app.mode = Mode::EditHex;
            app.hex_nibble = None;
        }
        _ => {}
    }
    false
}

fn handle_command(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.command_input.clear();
        }
        KeyCode::Enter => {
            return app.execute_command();
        }
        KeyCode::Backspace => {
            app.command_input.pop();
            if app.command_input.is_empty() {
                app.mode = Mode::Normal;
            }
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        _ => {}
    }
    false
}

fn handle_search(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.search_input.clear();
            app.search_results.clear();
            app.search_pattern_len = 0;
        }
        KeyCode::Enter => {
            app.mode = Mode::Normal;
            search::execute_search(app);
        }
        KeyCode::Backspace => {
            app.search_input.pop();
            if app.search_input.is_empty() {
                app.mode = Mode::Normal;
                app.search_results.clear();
                app.search_pattern_len = 0;
            } else {
                search::incremental_search(app);
            }
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
            search::incremental_search(app);
        }
        _ => {}
    }
    false
}

fn handle_strings(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Close panel and return to normal mode
        KeyCode::Esc | KeyCode::Char('q') => {
            app.strings_panel.visible = false;
            app.mode = Mode::Normal;
        }

        // Navigate down
        KeyCode::Char('j') | KeyCode::Down => {
            if !app.strings_panel.results.is_empty() {
                let max = app.strings_panel.results.len() - 1;
                if app.strings_panel.selected < max {
                    app.strings_panel.selected += 1;
                    app.strings_panel.ensure_selected_visible();
                }
            }
        }

        // Navigate up
        KeyCode::Char('k') | KeyCode::Up => {
            if app.strings_panel.selected > 0 {
                app.strings_panel.selected -= 1;
                app.strings_panel.ensure_selected_visible();
            }
        }

        // Page down
        KeyCode::PageDown => {
            if !app.strings_panel.results.is_empty() {
                let max = app.strings_panel.results.len() - 1;
                let step = app.strings_panel.visible_rows.saturating_sub(1).max(1);
                app.strings_panel.selected = (app.strings_panel.selected + step).min(max);
                app.strings_panel.ensure_selected_visible();
            }
        }

        // Page up
        KeyCode::PageUp => {
            let step = app.strings_panel.visible_rows.saturating_sub(1).max(1);
            app.strings_panel.selected = app.strings_panel.selected.saturating_sub(step);
            app.strings_panel.ensure_selected_visible();
        }

        // Jump to the selected string's offset in the hex view
        KeyCode::Enter => {
            if let Some(entry) = app.strings_panel.results.get(app.strings_panel.selected) {
                let offset = entry.offset;
                app.strings_panel.visible = false;
                app.mode = Mode::Normal;
                app.move_cursor_to(offset);
                app.status_message = Some(format!("Jumped to 0x{offset:08X}"));
            }
        }

        // Export strings to file
        KeyCode::Char('x') => {
            let path = std::path::Path::new("strings.txt");
            match crate::strings::export_strings(&app.strings_panel.results, path) {
                Ok(()) => {
                    let count = app.strings_panel.results.len();
                    app.status_message = Some(format!("Exported {count} strings to strings.txt"));
                }
                Err(e) => {
                    app.status_message = Some(format!("Export error: {e}"));
                }
            }
        }

        // Allow entering command mode while strings panel is open
        KeyCode::Char(':') => {
            app.mode = Mode::Command;
            app.command_input.clear();
        }

        _ => {}
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    use crossterm::event::MouseEventKind;
    use ratatui::prelude::Rect;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn key_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn make_app(data: &[u8]) -> App {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(data).unwrap();
        App::open(tmp.path().to_str().unwrap()).unwrap()
    }

    #[test]
    fn normal_vim_navigation() {
        let mut app = make_app(&vec![0u8; 256]);
        // l = right
        handle_key(&mut app, key(KeyCode::Char('l')));
        assert_eq!(app.cursor, 1);
        // h = left
        handle_key(&mut app, key(KeyCode::Char('h')));
        assert_eq!(app.cursor, 0);
        // j = down (1 row = 16 bytes)
        handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.cursor, 16);
        // k = up
        handle_key(&mut app, key(KeyCode::Char('k')));
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn normal_g_and_shift_g() {
        let mut app = make_app(&vec![0u8; 256]);
        handle_key(&mut app, key(KeyCode::Char('G')));
        assert_eq!(app.cursor, 255);
        handle_key(&mut app, key(KeyCode::Char('g')));
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn normal_q_does_not_quit() {
        let mut app = make_app(b"data");
        assert!(!handle_key(&mut app, key(KeyCode::Char('q'))));
    }

    #[test]
    fn enter_edit_hex_mode() {
        let mut app = make_app(b"test");
        handle_key(&mut app, key(KeyCode::Char('i')));
        assert_eq!(app.mode, Mode::EditHex);
    }

    #[test]
    fn enter_edit_ascii_mode() {
        let mut app = make_app(b"test");
        handle_key(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.mode, Mode::EditAscii);
    }

    #[test]
    fn enter_command_mode() {
        let mut app = make_app(b"test");
        handle_key(&mut app, key(KeyCode::Char(':')));
        assert_eq!(app.mode, Mode::Command);
    }

    #[test]
    fn enter_search_mode() {
        let mut app = make_app(b"test");
        handle_key(&mut app, key(KeyCode::Char('/')));
        assert_eq!(app.mode, Mode::Search);
    }

    #[test]
    fn edit_hex_write_byte() {
        let mut app = make_app(b"\x00\x00");
        app.mode = Mode::EditHex;
        // Type 'F' then 'F' → write 0xFF
        handle_key(&mut app, key(KeyCode::Char('F')));
        assert!(app.hex_nibble.is_some());
        handle_key(&mut app, key(KeyCode::Char('F')));
        assert_eq!(app.buffer.get(0), Some(0xFF));
        assert_eq!(app.cursor, 1); // auto-advanced
    }

    #[test]
    fn edit_hex_esc_returns_normal() {
        let mut app = make_app(b"test");
        app.mode = Mode::EditHex;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn edit_ascii_write_char() {
        let mut app = make_app(b"AB");
        app.mode = Mode::EditAscii;
        handle_key(&mut app, key(KeyCode::Char('Z')));
        assert_eq!(app.buffer.get(0), Some(b'Z'));
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn edit_hex_tab_switches_to_ascii() {
        let mut app = make_app(b"test");
        app.mode = Mode::EditHex;
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.mode, Mode::EditAscii);
    }

    #[test]
    fn edit_ascii_tab_switches_to_hex() {
        let mut app = make_app(b"test");
        app.mode = Mode::EditAscii;
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.mode, Mode::EditHex);
    }

    #[test]
    fn command_mode_typing_and_esc() {
        let mut app = make_app(b"test");
        app.mode = Mode::Command;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert_eq!(app.command_input, "q");
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.command_input.is_empty());
    }

    #[test]
    fn command_mode_backspace_exits_when_empty() {
        let mut app = make_app(b"test");
        app.mode = Mode::Command;
        handle_key(&mut app, key(KeyCode::Char('x')));
        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn search_mode_typing_and_enter() {
        let mut app = make_app(b"hello world");
        app.mode = Mode::Search;
        handle_key(&mut app, key(KeyCode::Char('w')));
        handle_key(&mut app, key(KeyCode::Char('o')));
        assert_eq!(app.search_input, "wo");
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.mode, Mode::Normal);
        // Search should have executed
        assert!(!app.search_results.is_empty());
    }

    #[test]
    fn ctrl_d_half_page_down() {
        let mut app = make_app(&vec![0u8; 4096]);
        app.visible_rows = 10;
        handle_key(&mut app, key_ctrl(KeyCode::Char('d')));
        assert_eq!(app.cursor, 5 * 16); // half of 10 rows
    }

    #[test]
    fn undo_in_normal_mode() {
        let mut app = make_app(b"\x00\x00");
        // Edit a byte
        app.mode = Mode::EditHex;
        handle_key(&mut app, key(KeyCode::Char('F')));
        handle_key(&mut app, key(KeyCode::Char('F')));
        assert_eq!(app.buffer.get(0), Some(0xFF));

        // Undo in normal mode
        app.mode = Mode::Normal;
        handle_key(&mut app, key(KeyCode::Char('u')));
        assert_eq!(app.buffer.get(0), Some(0x00));
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn redo_in_normal_mode() {
        let mut app = make_app(b"\x00\x00");
        app.mode = Mode::EditHex;
        handle_key(&mut app, key(KeyCode::Char('A')));
        handle_key(&mut app, key(KeyCode::Char('B')));
        assert_eq!(app.buffer.get(0), Some(0xAB));

        app.mode = Mode::Normal;
        handle_key(&mut app, key(KeyCode::Char('u')));
        assert_eq!(app.buffer.get(0), Some(0x00));

        handle_key(&mut app, key_ctrl(KeyCode::Char('r')));
        assert_eq!(app.buffer.get(0), Some(0xAB));
    }

    #[test]
    fn undo_nothing_shows_message() {
        let mut app = make_app(b"test");
        handle_key(&mut app, key(KeyCode::Char('u')));
        assert!(app.status_message.as_ref().unwrap().contains("Nothing to undo"));
    }

    #[test]
    fn enter_visual_mode() {
        let mut app = make_app(b"ABCDEF");
        app.cursor = 2;
        handle_key(&mut app, key(KeyCode::Char('v')));
        assert_eq!(app.mode, Mode::Visual);
        assert_eq!(app.selection_anchor, Some(2));
    }

    #[test]
    fn visual_mode_navigate_and_yank() {
        let mut app = make_app(b"ABCDEF");
        app.cursor = 1;
        // Enter visual mode
        handle_key(&mut app, key(KeyCode::Char('v')));
        assert_eq!(app.mode, Mode::Visual);
        // Extend selection right 2 times
        handle_key(&mut app, key(KeyCode::Char('l')));
        handle_key(&mut app, key(KeyCode::Char('l')));
        assert_eq!(app.cursor, 3);
        // Yank
        handle_key(&mut app, key(KeyCode::Char('y')));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.clipboard, vec![b'B', b'C', b'D']);
    }

    #[test]
    fn visual_mode_esc_cancels() {
        let mut app = make_app(b"ABCDEF");
        handle_key(&mut app, key(KeyCode::Char('v')));
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.selection_anchor.is_none());
    }

    #[test]
    fn paste_in_normal_mode() {
        let mut app = make_app(b"ABCDEF");
        app.clipboard = vec![b'X', b'Y'];
        app.cursor = 0;
        handle_key(&mut app, key(KeyCode::Char('p')));
        assert_eq!(app.buffer.get(0), Some(b'X'));
        assert_eq!(app.buffer.get(1), Some(b'Y'));
        assert!(app.status_message.as_ref().unwrap().contains("Pasted 2"));
    }

    #[test]
    fn paste_empty_shows_message() {
        let mut app = make_app(b"test");
        handle_key(&mut app, key(KeyCode::Char('p')));
        assert!(app.status_message.as_ref().unwrap().contains("Nothing to paste"));
    }

    #[test]
    fn set_bookmark_and_jump() {
        let mut app = make_app(&vec![0u8; 256]);
        app.cursor = 0x20;
        // m + a → set bookmark 'a' at 0x20
        handle_key(&mut app, key(KeyCode::Char('m')));
        assert!(app.pending_bookmark.is_some());
        handle_key(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.bookmarks.get(&'a'), Some(&0x20));
        assert!(app.status_message.as_ref().unwrap().contains("Bookmark 'a'"));

        // Move somewhere else
        app.move_cursor_to(0x80);
        assert_eq!(app.cursor, 0x80);

        // ' + a → jump back to 0x20
        handle_key(&mut app, key(KeyCode::Char('\'')));
        handle_key(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.cursor, 0x20);
        assert!(app.status_message.as_ref().unwrap().contains("Jumped to"));
    }

    #[test]
    fn jump_to_unset_bookmark() {
        let mut app = make_app(b"test");
        handle_key(&mut app, key(KeyCode::Char('\'')));
        handle_key(&mut app, key(KeyCode::Char('z')));
        assert!(app.status_message.as_ref().unwrap().contains("not set"));
    }

    #[test]
    fn bookmark_cancel_on_invalid_key() {
        let mut app = make_app(b"test");
        handle_key(&mut app, key(KeyCode::Char('m')));
        // Press a non-lowercase letter
        handle_key(&mut app, key(KeyCode::Char('1')));
        assert!(app.pending_bookmark.is_none());
        assert!(app.status_message.as_ref().unwrap().contains("cancelled"));
    }

    #[test]
    fn search_mode_esc_clears_results() {
        let mut app = make_app(b"hello world");
        app.mode = Mode::Search;
        handle_key(&mut app, key(KeyCode::Char('h')));
        assert!(!app.search_results.is_empty()); // incremental found 'h'
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.search_input.is_empty());
        assert!(app.search_results.is_empty());
        assert_eq!(app.search_pattern_len, 0);
    }

    #[test]
    fn search_mode_backspace_to_empty_exits() {
        let mut app = make_app(b"test");
        app.mode = Mode::Search;
        handle_key(&mut app, key(KeyCode::Char('t')));
        assert_eq!(app.mode, Mode::Search);
        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.search_results.is_empty());
    }

    #[test]
    fn search_mode_backspace_triggers_incremental() {
        let mut app = make_app(b"test data test");
        app.mode = Mode::Search;
        handle_key(&mut app, key(KeyCode::Char('t')));
        handle_key(&mut app, key(KeyCode::Char('e')));
        handle_key(&mut app, key(KeyCode::Char('s')));
        handle_key(&mut app, key(KeyCode::Char('t')));
        let count_4 = app.search_results.len();
        // Backspace to "tes" — should re-search incrementally
        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.search_input, "tes");
        assert_eq!(app.mode, Mode::Search);
        assert!(app.search_results.len() >= count_4);
    }

    #[test]
    fn normal_n_navigates_search_results() {
        let mut app = make_app(b"ABAB");
        // First do a search
        app.mode = Mode::Search;
        handle_key(&mut app, key(KeyCode::Char('A')));
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.search_results, vec![0, 2]);
        assert_eq!(app.cursor, 0);

        // n → next match
        handle_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.cursor, 2);

        // N → previous match
        handle_key(&mut app, key(KeyCode::Char('N')));
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn normal_home_end_navigation() {
        let mut app = make_app(&vec![0u8; 256]);
        app.cursor = 5;
        // 0 → start of row
        handle_key(&mut app, key(KeyCode::Char('0')));
        assert_eq!(app.cursor, 0);
        // $ → end of row
        handle_key(&mut app, key(KeyCode::Char('$')));
        assert_eq!(app.cursor, 15);
    }

    #[test]
    fn edit_hex_arrow_keys_clear_nibble() {
        let mut app = make_app(b"\x00\x00");
        app.mode = Mode::EditHex;
        // Type first nibble
        handle_key(&mut app, key(KeyCode::Char('A')));
        assert!(app.hex_nibble.is_some());
        // Arrow clears nibble and moves
        handle_key(&mut app, key(KeyCode::Right));
        assert!(app.hex_nibble.is_none());
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn visual_mode_home_end_g_G() {
        let mut app = make_app(&vec![0u8; 256]);
        app.cursor = 5;
        handle_key(&mut app, key(KeyCode::Char('v')));
        assert_eq!(app.mode, Mode::Visual);

        // Home-equivalent: 0
        handle_key(&mut app, key(KeyCode::Char('0')));
        assert_eq!(app.cursor, 0);

        // End-equivalent: $
        handle_key(&mut app, key(KeyCode::Char('$')));
        assert_eq!(app.cursor, 15);

        // g → beginning of file
        handle_key(&mut app, key(KeyCode::Char('g')));
        assert_eq!(app.cursor, 0);

        // G → end of file
        handle_key(&mut app, key(KeyCode::Char('G')));
        assert_eq!(app.cursor, 255);

        // Still in visual mode with anchor
        assert_eq!(app.mode, Mode::Visual);
        assert!(app.selection_anchor.is_some());
    }

    #[test]
    fn multiple_bookmarks() {
        let mut app = make_app(&vec![0u8; 256]);
        // Set bookmark 'a' at 0x10
        app.cursor = 0x10;
        handle_key(&mut app, key(KeyCode::Char('m')));
        handle_key(&mut app, key(KeyCode::Char('a')));

        // Set bookmark 'b' at 0x50
        app.move_cursor_to(0x50);
        handle_key(&mut app, key(KeyCode::Char('m')));
        handle_key(&mut app, key(KeyCode::Char('b')));

        assert_eq!(app.bookmarks.len(), 2);

        // Jump to 'a'
        handle_key(&mut app, key(KeyCode::Char('\'')));
        handle_key(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.cursor, 0x10);

        // Jump to 'b'
        handle_key(&mut app, key(KeyCode::Char('\'')));
        handle_key(&mut app, key(KeyCode::Char('b')));
        assert_eq!(app.cursor, 0x50);
    }

    #[test]
    fn strings_mode_esc_closes_panel() {
        let mut app = make_app(b"Hello World this is a test");
        app.mode = Mode::Strings;
        app.strings_panel.visible = true;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Normal);
        assert!(!app.strings_panel.visible);
    }

    #[test]
    fn strings_mode_navigate_down_up() {
        use crate::strings::{StringEntry, StringKind};
        let mut app = make_app(b"test");
        app.mode = Mode::Strings;
        app.strings_panel.visible = true;
        app.strings_panel.results = vec![
            StringEntry { offset: 0, length: 4, kind: StringKind::Ascii, text: "aaaa".to_string() },
            StringEntry { offset: 10, length: 4, kind: StringKind::Ascii, text: "bbbb".to_string() },
            StringEntry { offset: 20, length: 4, kind: StringKind::Ascii, text: "cccc".to_string() },
        ];
        app.strings_panel.selected = 0;

        handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.strings_panel.selected, 1);

        handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.strings_panel.selected, 2);

        // Can't go past end
        handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.strings_panel.selected, 2);

        handle_key(&mut app, key(KeyCode::Char('k')));
        assert_eq!(app.strings_panel.selected, 1);
    }

    #[test]
    fn strings_mode_enter_jumps_to_offset() {
        use crate::strings::{StringEntry, StringKind};
        let mut app = make_app(&vec![0u8; 256]);
        app.mode = Mode::Strings;
        app.strings_panel.visible = true;
        app.strings_panel.results = vec![
            StringEntry { offset: 0x40, length: 4, kind: StringKind::Ascii, text: "test".to_string() },
        ];
        app.strings_panel.selected = 0;

        handle_key(&mut app, key(KeyCode::Enter));

        assert_eq!(app.mode, Mode::Normal);
        assert!(!app.strings_panel.visible);
        assert_eq!(app.cursor, 0x40);
        assert!(app.status_message.as_ref().unwrap().contains("0x00000040"));
    }

    #[test]
    fn strings_mode_q_closes_panel() {
        let mut app = make_app(b"test");
        app.mode = Mode::Strings;
        app.strings_panel.visible = true;
        handle_key(&mut app, key(KeyCode::Char('q')));
        assert_eq!(app.mode, Mode::Normal);
        assert!(!app.strings_panel.visible);
    }

    fn mouse_event(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn make_app_with_area(data: &[u8]) -> App {
        let mut app = make_app(data);
        // Simulate hex view area as if terminal is at (0,0) with header at row 0
        // Hex view inner area starts at row 1
        app.hex_view_area = Rect::new(0, 1, 80, 20);
        app.visible_rows = 20;
        app
    }

    #[test]
    fn mouse_click_positions_cursor_hex() {
        let mut app = make_app_with_area(&vec![0u8; 256]);
        // Hex byte 0 is at x = 9 + 1 = 10 (offset col 9 chars + 1 leading space)
        handle_mouse(&mut app, mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 1));
        assert_eq!(app.cursor, 0);

        // Hex byte 1 is at x = 9 + 4 = 13
        handle_mouse(&mut app, mouse_event(MouseEventKind::Down(MouseButton::Left), 13, 1));
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn mouse_click_positions_cursor_ascii() {
        let mut app = make_app_with_area(&vec![0u8; 256]);
        // ASCII section starts at x = 9 + (16*3+2) + 1 = 60, first char is "│" at 60
        // Byte 0 is at x = 61
        handle_mouse(&mut app, mouse_event(MouseEventKind::Down(MouseButton::Left), 61, 1));
        assert_eq!(app.cursor, 0);

        // Byte 3 is at x = 64
        handle_mouse(&mut app, mouse_event(MouseEventKind::Down(MouseButton::Left), 64, 1));
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn mouse_click_row_offset() {
        let mut app = make_app_with_area(&vec![0u8; 256]);
        // Click byte 0 on row 2 (data row 1) → offset 16
        handle_mouse(&mut app, mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 2));
        assert_eq!(app.cursor, 16);
    }

    #[test]
    fn mouse_drag_starts_visual_selection() {
        let mut app = make_app_with_area(&vec![0u8; 256]);
        // Click at byte 0
        handle_mouse(&mut app, mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 1));
        assert_eq!(app.cursor, 0);
        assert_eq!(app.mode, Mode::Normal);

        // Drag to byte 3
        handle_mouse(&mut app, mouse_event(MouseEventKind::Drag(MouseButton::Left), 19, 1));
        assert_eq!(app.mode, Mode::Visual);
        assert_eq!(app.selection_anchor, Some(0));
        assert_eq!(app.cursor, 3);
        assert_eq!(app.selection_range(), Some((0, 3)));
    }

    #[test]
    fn mouse_scroll_moves_cursor() {
        let mut app = make_app_with_area(&vec![0u8; 4096]);
        assert_eq!(app.cursor, 0);
        // Scroll down
        handle_mouse(&mut app, mouse_event(MouseEventKind::ScrollDown, 0, 0));
        assert_eq!(app.cursor, 3 * 16); // 3 lines * 16 bytes

        // Scroll up
        handle_mouse(&mut app, mouse_event(MouseEventKind::ScrollUp, 0, 0));
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn mouse_click_exits_command_mode() {
        let mut app = make_app_with_area(&vec![0u8; 256]);
        app.mode = Mode::Command;
        app.command_input = "goto".to_string();
        handle_mouse(&mut app, mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 1));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn mouse_click_outside_hex_view_ignored() {
        let mut app = make_app_with_area(&vec![0u8; 256]);
        app.cursor = 5;
        // Click in offset column (x=2) — should not move cursor
        handle_mouse(&mut app, mouse_event(MouseEventKind::Down(MouseButton::Left), 2, 1));
        assert_eq!(app.cursor, 5);
    }

    #[test]
    fn ctrl_u_half_page_up() {
        let mut app = make_app(&vec![0u8; 4096]);
        app.visible_rows = 10;
        // Move down first
        app.move_cursor_to(5 * 16);
        // Ctrl-U should move up half a page (5 rows * 16 bytes)
        handle_key(&mut app, key_ctrl(KeyCode::Char('u')));
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn page_down_and_page_up() {
        let mut app = make_app(&vec![0u8; 4096]);
        app.visible_rows = 4;
        // PageDown moves a full page (4 rows * 16 bytes = 64 bytes)
        handle_key(&mut app, key(KeyCode::PageDown));
        assert_eq!(app.cursor, 4 * 16);
        // PageUp returns to start
        handle_key(&mut app, key(KeyCode::PageUp));
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn quit_dirty_buffer_blocked() {
        let mut app = make_app(b"\x00");
        // Write a byte to mark buffer dirty
        app.mode = Mode::EditHex;
        handle_key(&mut app, key(KeyCode::Char('F')));
        handle_key(&mut app, key(KeyCode::Char('F')));
        app.mode = Mode::Normal;

        // :q on dirty buffer must NOT quit
        handle_key(&mut app, key(KeyCode::Char(':')));
        handle_key(&mut app, key(KeyCode::Char('q')));
        let quit = handle_key(&mut app, key(KeyCode::Enter));
        assert!(!quit);
        assert!(app
            .status_message
            .as_ref()
            .unwrap()
            .contains("Unsaved changes"));
    }

    #[test]
    fn quit_force_dirty_buffer() {
        let mut app = make_app(b"\x00");
        // Dirty the buffer
        app.mode = Mode::EditHex;
        handle_key(&mut app, key(KeyCode::Char('A')));
        handle_key(&mut app, key(KeyCode::Char('B')));
        app.mode = Mode::Normal;

        // :q! must quit even with unsaved changes
        handle_key(&mut app, key(KeyCode::Char(':')));
        for c in "q!".chars() {
            handle_key(&mut app, key(KeyCode::Char(c)));
        }
        let quit = handle_key(&mut app, key(KeyCode::Enter));
        assert!(quit);
    }

    #[test]
    fn edit_ascii_non_printable_rejected() {
        let mut app = make_app(b"\x00");
        app.mode = Mode::EditAscii;
        // Ctrl+C key event (Char 'c' + CONTROL modifier) must NOT write
        let ctrl_c = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        handle_key(&mut app, ctrl_c);
        assert_eq!(app.buffer.get(0), Some(0x00)); // unchanged
        assert_eq!(app.cursor, 0); // cursor did not advance
    }
}
