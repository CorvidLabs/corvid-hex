---
module: chx
version: 1
status: draft
files:
  - src/main.rs
  - src/app.rs
  - src/strings.rs
db_tables: []
depends_on:
  - specs/buffer/buffer.spec.md
  - specs/input/input.spec.md
  - specs/render/render.spec.md
  - specs/search/search.spec.md
---

## Purpose

Core application module for the `chx` hex editor. Defines the application state machine (`App`), the mode enum (`Mode`), command execution, cursor navigation, and the main entry point with terminal setup/teardown and event loop.

## Public API

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `Mode` | `pub enum Mode { Normal, Visual, EditHex, EditAscii, Command, Search, Strings }` | Represents the current editor mode. Each variant controls input dispatch and UI rendering. |
| `label` | `pub fn label(&self) -> &'static str` | Returns the display string for the mode (e.g., "NORMAL", "EDIT-HEX"). |
| `App` | `pub struct App` | Central application state. Holds buffer, mode, cursor, scroll offset, search state, command input, hex nibble tracking. Public fields include `bookmarks: HashMap<char, usize>` for named offset bookmarks (a-z) and `pending_bookmark: Option<char>` for two-key bookmark commands. |
| `open` | `pub fn open(path: &str) -> Result<Self>` | Creates a new App by opening a file into a Buffer. Initializes all state to defaults (Normal mode, cursor at 0, 16 bytes per row). |
| `cursor_row` | `pub fn cursor_row(&self) -> usize` | Returns the row index of the current cursor position. |
| `ensure_cursor_visible` | `pub fn ensure_cursor_visible(&mut self)` | Adjusts `scroll_offset` so the cursor row is within the visible viewport. |
| `move_cursor` | `pub fn move_cursor(&mut self, offset: isize)` | Moves cursor by a signed offset, clamped to `[0, len-1]`. Calls `ensure_cursor_visible`. |
| `move_cursor_to` | `pub fn move_cursor_to(&mut self, pos: usize)` | Moves cursor to an absolute position, clamped to `[0, len-1]`. Calls `ensure_cursor_visible`. |
| `page_down` | `pub fn page_down(&mut self)` | Moves cursor forward by one page (`visible_rows * bytes_per_row`). |
| `page_up` | `pub fn page_up(&mut self)` | Moves cursor backward by one page. |
| `selection_range` | `pub fn selection_range(&self) -> Option<(usize, usize)>` | Returns the selected byte range (lo, hi) inclusive if in visual mode, or `None` if no selection anchor is set. |
| `yank_selection` | `pub fn yank_selection(&mut self) -> usize` | Copies selected bytes into the clipboard and clears the selection anchor. Returns the number of bytes yanked (0 if no selection). |
| `paste` | `pub fn paste(&mut self) -> usize` | Overwrites bytes at cursor with clipboard contents (clamped to buffer length). Returns the number of bytes pasted. |
| `execute_command` | `pub fn execute_command(&mut self) -> bool` | Parses and executes the current command input. Returns true if the app should quit. Supports `:q`, `:q!`, `:w`, `:wq`, `:goto`/`:g`, `:s/find/replace`, `:columns`/`:cols`, `:marks`, `:strings`. |
| `offset_from_screen` | `pub fn offset_from_screen(&self, x: u16, y: u16) -> Option<usize>` | Maps terminal (x, y) screen coordinates to a byte offset. Returns `None` if the click is outside the hex view area or beyond the buffer length. Used for mouse click-to-position. |
| `StringsPanel` | `pub struct StringsPanel` | State for the strings extraction panel. Contains `visible`, `results: Vec<StringEntry>`, `selected`, `scroll`, `min_length`, and `visible_rows` fields. |
| `new` (StringsPanel) | `pub fn new() -> Self` | Creates a new `StringsPanel` with default values (not visible, empty results, min_length 4). |
| `ensure_selected_visible` | `pub fn ensure_selected_visible(&mut self)` | Adjusts `scroll` so the `selected` entry is within the visible viewport of the strings panel. |
| `StringEntry` | `pub struct StringEntry` | Represents an extracted string with `offset`, `length`, `kind: StringKind`, and `text` fields. |
| `StringKind` | `pub enum StringKind { Ascii, Utf8, Utf16Le, Utf16Be }` | Classification of an extracted string's encoding. |
| `extract_strings` | `pub fn extract_strings(data: &[u8], min_length: usize) -> Vec<StringEntry>` | Scans binary data for ASCII, UTF-8, UTF-16 LE, and UTF-16 BE strings of at least `min_length` characters. Returns results sorted by offset. |
| `export_strings` | `pub fn export_strings(entries: &[StringEntry], path: &Path) -> io::Result<()>` | Writes string entries to a text file in tab-separated format (`offset\tkind\ttext`). |

## Invariants

1. `bytes_per_row` defaults to 16 and can be changed via the `:columns` command.
2. Cursor is always clamped to `[0, buffer.len() - 1]` (or 0 for empty buffers).
3. `ensure_cursor_visible` guarantees `scroll_offset <= cursor_row < scroll_offset + visible_rows`.
4. `execute_command` always clears `command_input` and returns to Normal mode before processing the command.
5. `:q` with a dirty buffer refuses to quit and sets a warning status message.
6. `:q!` always quits regardless of dirty state.
7. `:wq` only quits if the save succeeds; on save error, it stays open with an error message.
8. Terminal raw mode and alternate screen are always restored on exit, even on error (cleanup runs unconditionally after `run()`).
9. Bookmarks are stored per-session in memory (not persisted to disk).
10. `pending_bookmark` is consumed (taken) at the start of the next key event. Invalid follow-up keys cancel the operation.

## Behavioral Examples

**Normal startup**
- Given: a valid file path is provided as CLI arg
- When: `chx somefile.bin` is run
- Then: terminal enters raw mode + alternate screen, file is loaded, UI renders in Normal mode

**Goto command**
- Given: mode is Command, command_input is `"goto 0xFF"`
- When: `execute_command` is called
- Then: cursor moves to offset 255, status message says "Jumped to 0x000000FF"

**Invalid goto address**
- Given: command_input is `"goto xyz"`
- When: `execute_command` is called
- Then: status message says "Invalid address: xyz", cursor doesn't move

**Save and quit**
- Given: buffer has unsaved edits
- When: `:wq` command is executed
- Then: buffer is saved to disk, then app quits

**Unknown command**
- Given: command_input is `"foo"`
- When: `execute_command` is called
- Then: status message says "Unknown command: foo"

**List bookmarks**
- Given: bookmarks 'a' at 0x10, 'b' at 0x20
- When: `:marks` command is executed
- Then: status message shows "Marks: a:0x10 b:0x20" (sorted alphabetically)

**List bookmarks (empty)**
- Given: no bookmarks are set
- When: `:marks` command is executed
- Then: status message shows "No bookmarks set"

## Error Cases

| Condition | Behavior |
|-----------|----------|
| File cannot be opened | `App::open` returns `Err`, main exits with error message |
| `:q` with dirty buffer | Quit blocked, status shows "Unsaved changes! Use :q! to force quit" |
| `:w` fails (disk error) | Status shows "Error: {details}", app continues |
| `:wq` save fails | Status shows error, quit is cancelled |
| Invalid goto address | Status shows "Invalid address: {input}" |
| Terminal setup fails | `main` returns error immediately |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `crate::buffer::Buffer` | File I/O and edit overlay |
| `crate::input` | `handle_key` for keyboard dispatch |
| `crate::render` | `draw` for TUI rendering |
| `crate::search` | Search execution (via input handlers) |
| `crate::strings` | String extraction and export |
| `clap` | CLI argument parsing (`Parser` derive) |
| `crossterm` | Terminal raw mode, alternate screen, key events |
| `ratatui` | TUI framework (`Terminal`, `Frame`, `CrosstermBackend`) |
| `anyhow` | Error handling |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-29 | Initial spec |
| 2026-03-29 | Add Visual mode, selection_range, yank_selection, paste exports |
| 2026-03-30 | Add bookmarks (HashMap), pending_bookmark, :marks command, :s/find/replace, :columns/:cols |
| 2026-03-30 | Add offset_from_screen export for mouse coordinate mapping |
| 2026-03-30 | Add strings extraction: StringsPanel, StringEntry, StringKind, extract_strings, export_strings, Strings mode |
