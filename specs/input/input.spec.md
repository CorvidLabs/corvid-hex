---
module: input
version: 1
status: draft
files:
  - src/input.rs
db_tables: []
depends_on: []
---

## Purpose

Translates keyboard events into application actions for a terminal hex editor. Acts as the central input dispatcher, routing key events to mode-specific handlers (Normal, EditHex, EditAscii, Command, Search) and returning a quit signal when appropriate.

## Public API

### Exported Functions

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `handle_key` | `fn handle_key(app: &mut App, key: KeyEvent) -> bool` | Dispatches a key event to the handler for the current app mode. Returns `true` if the application should quit. |
| `handle_mouse` | `pub fn handle_mouse(app: &mut App, mouse: MouseEvent)` | Handles mouse events: left-click positions the cursor, drag starts/extends visual selection, scroll wheel moves by 3 rows. Exits text-input modes on click. |

## Invariants

1. `handle_key` always returns `false` except when a command triggers quit (`:q`, `:q!`, `:wq`).
2. Pressing `q` in Normal mode does not quit — quit is only available through the command system (`:q`/`:q!`).
3. Entering EditHex mode always clears `hex_nibble` to `None`.
4. Arrow-key navigation in EditHex mode resets `hex_nibble` to `None`, discarding any partial nibble input.
5. Tab toggles between EditHex and EditAscii modes; switching to EditHex via Tab resets `hex_nibble`.
6. In EditHex mode, a full byte is written only after two consecutive valid hex digit inputs (high nibble then low nibble), after which the cursor advances by one.
7. In Command and Search modes, if Backspace empties the input buffer, the mode reverts to Normal.
8. EditAscii mode only accepts non-control ASCII characters (`is_ascii()` and no `CONTROL` modifier).
9. Entering Normal mode from any sub-handler clears mode-specific state (nibble buffer, command input, or search input as applicable).
10. Two-key bookmark sequences (`m`+letter, `'`+letter) are handled at the top of `handle_normal` before the main match — `pending_bookmark` is consumed first.
11. Only lowercase a-z are valid bookmark names; any other follow-up key cancels the pending operation.

## Behavioral Examples

**Normal mode q does not quit**
- Given: app is in Normal mode
- When: user presses `q`
- Then: `handle_key` returns `false` (quit only via `:q` command)

**Hex editing two-nibble write**
- Given: app is in EditHex mode with `hex_nibble` as `None`, cursor at position 5
- When: user presses `a` then `3`
- Then: byte `0xA3` is written at position 5, cursor moves to position 6, `hex_nibble` resets to `None`

**Hex editing partial nibble discarded on navigation**
- Given: app is in EditHex mode with one nibble entered (`hex_nibble` is `Some(0xA)`)
- When: user presses Left arrow
- Then: `hex_nibble` resets to `None` and cursor moves left by 1

**Tab toggles between hex and ASCII edit**
- Given: app is in EditHex mode
- When: user presses Tab
- Then: mode switches to EditAscii and `hex_nibble` resets to `None`

**Search execution**
- Given: app is in Search mode with `search_input` containing `"FF"`
- When: user presses Enter
- Then: mode returns to Normal and `search::execute_search` is called

**Command backspace exits on empty**
- Given: app is in Command mode with `command_input` containing `"w"`
- When: user presses Backspace twice
- Then: first Backspace removes `'w'`, second finds input empty and reverts mode to Normal

**Vi-style navigation in Normal mode**
- Given: app is in Normal mode with `bytes_per_row` of 16
- When: user presses `j`
- Then: cursor moves forward by 16 positions

**Set bookmark**
- Given: app is in Normal mode, cursor at 0x20
- When: user presses `m` then `a`
- Then: bookmark 'a' is set at offset 0x20, status shows "Bookmark 'a' set at 0x20"

**Jump to bookmark**
- Given: bookmark 'a' is set at 0x20, cursor elsewhere
- When: user presses `'` then `a`
- Then: cursor moves to 0x20, status shows "Jumped to bookmark 'a'"

**Jump to unset bookmark**
- Given: no bookmark 'z' exists
- When: user presses `'` then `z`
- Then: status shows "Bookmark 'z' not set", cursor unchanged

**Bookmark cancelled by invalid key**
- Given: user pressed `m` (pending_bookmark is set)
- When: user presses `1` (non-lowercase letter)
- Then: pending operation cancelled, status shows "Bookmark cancelled"

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `q` pressed in Normal mode | Key is ignored; quit requires `:q` or `:q!` command |
| Non-hex character pressed in EditHex mode | Key event is ignored (no state change) |
| Control character pressed in EditAscii mode | Key event is ignored |
| Unrecognized key in any mode | Key event is silently ignored |
| `G` pressed on an empty buffer in Normal mode | No cursor movement occurs (guarded by `is_empty()` check) |

## Dependencies

| Module | Symbols Used | Purpose |
|--------|-------------|---------|
| `crate::app` | `App`, `Mode` | Application state and mode enum |
| `crate::search` | `next_search_result`, `prev_search_result`, `execute_search` | Search execution and result navigation |
| `crossterm::event` | `KeyCode`, `KeyEvent`, `KeyModifiers`, `MouseEvent`, `MouseEventKind`, `MouseButton` | Keyboard and mouse event types |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-29 | Initial spec |
| 2026-03-30 | Add bookmark two-key sequences (m+letter, '+letter), Visual mode handler |
| 2026-03-30 | Add handle_mouse export for mouse click, drag, and scroll support |
