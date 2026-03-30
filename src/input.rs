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
