use crate::app::{App, Mode};
use crate::search;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a key event. Returns true if the app should quit.
pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match app.mode {
        Mode::Normal => handle_normal(app, key),
        Mode::EditHex => handle_edit_hex(app, key),
        Mode::EditAscii => handle_edit_ascii(app, key),
        Mode::Command => handle_command(app, key),
        Mode::Search => handle_search(app, key),
    }
}

fn handle_normal(app: &mut App, key: KeyEvent) -> bool {
    app.status_message = None;

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

        // Search navigation
        KeyCode::Char('n') => search::next_search_result(app),
        KeyCode::Char('N') => search::prev_search_result(app),

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
            let nibble = u8::from_str_radix(&c.to_string(), 16).unwrap();
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
        }
        KeyCode::Enter => {
            app.mode = Mode::Normal;
            search::execute_search(app);
        }
        KeyCode::Backspace => {
            app.search_input.pop();
            if app.search_input.is_empty() {
                app.mode = Mode::Normal;
            }
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
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
}
