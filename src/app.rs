use crate::buffer::Buffer;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    EditHex,
    EditAscii,
    Command,
    Search,
}

impl Mode {
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::EditHex => "EDIT-HEX",
            Mode::EditAscii => "EDIT-ASCII",
            Mode::Command => "COMMAND",
            Mode::Search => "SEARCH",
        }
    }
}

pub struct App {
    pub buffer: Buffer,
    pub mode: Mode,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub bytes_per_row: usize,
    pub visible_rows: usize,
    pub command_input: String,
    pub search_input: String,
    pub status_message: Option<String>,
    /// When editing hex, tracks whether we're on the high or low nibble.
    pub hex_nibble: Option<u8>,
    pub search_results: Vec<usize>,
    pub search_index: usize,
}

impl App {
    pub fn open(path: &str) -> Result<Self> {
        let buffer = Buffer::open(path)?;
        Ok(Self {
            buffer,
            mode: Mode::Normal,
            cursor: 0,
            scroll_offset: 0,
            bytes_per_row: 16,
            visible_rows: 24,
            command_input: String::new(),
            search_input: String::new(),
            status_message: None,
            hex_nibble: None,
            search_results: Vec::new(),
            search_index: 0,
        })
    }

    pub fn cursor_row(&self) -> usize {
        self.cursor / self.bytes_per_row
    }

    pub fn ensure_cursor_visible(&mut self) {
        let row = self.cursor_row();
        if row < self.scroll_offset {
            self.scroll_offset = row;
        } else if row >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = row - self.visible_rows + 1;
        }
    }

    pub fn move_cursor(&mut self, offset: isize) {
        let new_pos = self.cursor as isize + offset;
        let max = if self.buffer.is_empty() {
            0
        } else {
            self.buffer.len() - 1
        };
        self.cursor = new_pos.clamp(0, max as isize) as usize;
        self.ensure_cursor_visible();
    }

    pub fn move_cursor_to(&mut self, pos: usize) {
        let max = if self.buffer.is_empty() {
            0
        } else {
            self.buffer.len() - 1
        };
        self.cursor = pos.min(max);
        self.ensure_cursor_visible();
    }

    pub fn page_down(&mut self) {
        let jump = self.visible_rows * self.bytes_per_row;
        self.move_cursor(jump as isize);
    }

    pub fn page_up(&mut self) {
        let jump = self.visible_rows * self.bytes_per_row;
        self.move_cursor(-(jump as isize));
    }

    pub fn execute_command(&mut self) -> bool {
        let cmd = self.command_input.trim().to_string();
        self.command_input.clear();
        self.mode = Mode::Normal;

        match cmd.as_str() {
            "q" => {
                if self.buffer.is_dirty() {
                    self.status_message =
                        Some("Unsaved changes! Use :q! to force quit".to_string());
                    return false;
                }
                return true;
            }
            "q!" => return true,
            "w" => match self.buffer.save() {
                Ok(()) => {
                    self.status_message = Some("Written".to_string());
                }
                Err(e) => {
                    self.status_message = Some(format!("Error: {e}"));
                }
            },
            "wq" => {
                if let Err(e) = self.buffer.save() {
                    self.status_message = Some(format!("Error: {e}"));
                    return false;
                }
                return true;
            }
            _ if cmd.starts_with("goto ") || cmd.starts_with("g ") => {
                let addr_str = cmd.split_whitespace().nth(1).unwrap_or("");
                let addr_str = addr_str.trim_start_matches("0x").trim_start_matches("0X");
                match usize::from_str_radix(addr_str, 16) {
                    Ok(addr) => {
                        self.move_cursor_to(addr);
                        self.status_message = Some(format!("Jumped to 0x{addr:08X}"));
                    }
                    Err(_) => {
                        self.status_message = Some(format!("Invalid address: {}", cmd.split_whitespace().nth(1).unwrap_or("")));
                    }
                }
            }
            _ => {
                self.status_message = Some(format!("Unknown command: {cmd}"));
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_app(data: &[u8]) -> App {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(data).unwrap();
        App::open(tmp.path().to_str().unwrap()).unwrap()
    }

    #[test]
    fn initial_state() {
        let app = make_app(b"Hello");
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.cursor, 0);
        assert_eq!(app.scroll_offset, 0);
        assert_eq!(app.bytes_per_row, 16);
        assert!(!app.buffer.is_dirty());
    }

    #[test]
    fn move_cursor_forward_back() {
        let mut app = make_app(b"ABCDEF");
        app.move_cursor(3);
        assert_eq!(app.cursor, 3);
        app.move_cursor(-1);
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn move_cursor_clamps_at_zero() {
        let mut app = make_app(b"ABCD");
        app.move_cursor(-10);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn move_cursor_clamps_at_end() {
        let mut app = make_app(b"ABCD");
        app.move_cursor(100);
        assert_eq!(app.cursor, 3); // len-1
    }

    #[test]
    fn move_cursor_to_clamps() {
        let mut app = make_app(b"ABCD");
        app.move_cursor_to(999);
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn ensure_cursor_visible_scrolls_down() {
        let mut app = make_app(&vec![0u8; 1024]);
        app.visible_rows = 4;
        app.bytes_per_row = 16;
        // Move to row 10
        app.move_cursor_to(10 * 16);
        assert!(app.scroll_offset <= 10);
        assert!(app.scroll_offset + app.visible_rows > 10);
    }

    #[test]
    fn page_down_up() {
        let mut app = make_app(&vec![0u8; 4096]);
        app.visible_rows = 4;
        app.page_down();
        assert_eq!(app.cursor, 4 * 16);
        app.page_up();
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn execute_command_quit_clean() {
        let mut app = make_app(b"test");
        app.command_input = "q".to_string();
        assert!(app.execute_command());
    }

    #[test]
    fn execute_command_quit_dirty_blocked() {
        let mut app = make_app(b"test");
        app.buffer.set(0, 0xFF);
        app.command_input = "q".to_string();
        assert!(!app.execute_command());
        assert!(app.status_message.as_ref().unwrap().contains("Unsaved"));
    }

    #[test]
    fn execute_command_force_quit() {
        let mut app = make_app(b"test");
        app.buffer.set(0, 0xFF);
        app.command_input = "q!".to_string();
        assert!(app.execute_command());
    }

    #[test]
    fn execute_command_goto() {
        let mut app = make_app(&vec![0u8; 256]);
        app.command_input = "goto 0x10".to_string();
        app.execute_command();
        assert_eq!(app.cursor, 0x10);
    }

    #[test]
    fn execute_command_goto_short() {
        let mut app = make_app(&vec![0u8; 256]);
        app.command_input = "g 20".to_string();
        app.execute_command();
        assert_eq!(app.cursor, 0x20);
    }

    #[test]
    fn execute_command_unknown() {
        let mut app = make_app(b"test");
        app.command_input = "foobar".to_string();
        app.execute_command();
        assert!(app.status_message.as_ref().unwrap().contains("Unknown"));
    }

    #[test]
    fn execute_command_write() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"test").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();
        app.buffer.set(0, b'T');
        app.command_input = "w".to_string();
        app.execute_command();
        assert!(!app.buffer.is_dirty());
        assert!(app.status_message.as_ref().unwrap().contains("Written"));
    }

    #[test]
    fn execute_command_wq() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"test").unwrap();
        let mut app = App::open(tmp.path().to_str().unwrap()).unwrap();
        app.command_input = "wq".to_string();
        assert!(app.execute_command());
    }

    #[test]
    fn mode_labels() {
        assert_eq!(Mode::Normal.label(), "NORMAL");
        assert_eq!(Mode::EditHex.label(), "EDIT-HEX");
        assert_eq!(Mode::EditAscii.label(), "EDIT-ASCII");
        assert_eq!(Mode::Command.label(), "COMMAND");
        assert_eq!(Mode::Search.label(), "SEARCH");
    }
}
