use crate::app::{App, Mode};
use crate::search;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a key event. Returns true if the app should quit.
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match app.mode {
        Mode::Normal => handle_normal(app, key),
        Mode::Visual => handle_visual(app, key),
        Mode::EditHex => handle_edit_hex(app, key),
        Mode::EditAscii => handle_edit_ascii(app, key),
        Mode::Command => handle_command(app, key),
        Mode::Search => handle_search(app, key),
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
        // Quit
        KeyCode::Char('q') => {
            if app.buffer.is_dirty() {
                app.status_message =
                    Some("Unsaved changes! Use :q! to force quit".to_string());
                return false;
            }
            return true;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

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
    fn normal_q_clean_quits() {
        let mut app = make_app(b"data");
        assert!(handle_key(&mut app, key(KeyCode::Char('q'))));
    }

    #[test]
    fn normal_q_dirty_blocks() {
        let mut app = make_app(b"data");
        app.buffer.set(0, 0xFF);
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
}
