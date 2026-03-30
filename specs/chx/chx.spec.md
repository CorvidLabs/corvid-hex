---
module: chx
version: 1
status: draft
files:
  - src/main.rs
  - src/app.rs
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
| `Mode` | `pub enum Mode { Normal, Visual, EditHex, EditAscii, Command, Search }` | Represents the current editor mode. Each variant controls input dispatch and UI rendering. |
| `label` | `pub fn label(&self) -> &'static str` | Returns the display string for the mode (e.g., "NORMAL", "EDIT-HEX"). |
| `App` | `pub struct App` | Central application state. Holds buffer, mode, cursor, scroll offset, search state, command input, and hex nibble tracking. |
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
| `execute_command` | `pub fn execute_command(&mut self) -> bool` | Parses and executes the current command input. Returns true if the app should quit. Supports `:q`, `:q!`, `:w`, `:wq`, `:goto`/`:g`. |

## Invariants

1. `bytes_per_row` defaults to 16 and can be changed via the `:columns` command.
2. Cursor is always clamped to `[0, buffer.len() - 1]` (or 0 for empty buffers).
3. `ensure_cursor_visible` guarantees `scroll_offset <= cursor_row < scroll_offset + visible_rows`.
4. `execute_command` always clears `command_input` and returns to Normal mode before processing the command.
5. `:q` with a dirty buffer refuses to quit and sets a warning status message.
6. `:q!` always quits regardless of dirty state.
7. `:wq` only quits if the save succeeds; on save error, it stays open with an error message.
8. Terminal raw mode and alternate screen are always restored on exit, even on error (cleanup runs unconditionally after `run()`).

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
| `clap` | CLI argument parsing (`Parser` derive) |
| `crossterm` | Terminal raw mode, alternate screen, key events |
| `ratatui` | TUI framework (`Terminal`, `Frame`, `CrosstermBackend`) |
| `anyhow` | Error handling |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-29 | Initial spec |
| 2026-03-29 | Add Visual mode, selection_range, yank_selection, paste exports |
