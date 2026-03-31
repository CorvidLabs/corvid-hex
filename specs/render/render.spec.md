---
module: render
version: 1
status: draft
files:
  - src/render.rs
db_tables: []
depends_on: []
---

## Purpose

Renders the terminal UI for the hex editor. This module owns the full-screen layout — a header bar showing file metadata, a scrollable hex/ASCII view with syntax coloring and cursor highlighting, an optional data inspector panel, and a modal status bar — and translates `App` state into styled `ratatui` widgets each frame.

## Public API

### Exported Functions

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `draw` | `pub fn draw(f: &mut Frame, app: &mut App)` | Entry point for rendering a single frame. Splits the terminal area into header, hex view, and status bar regions, updates `app.visible_rows` to match the current terminal height, and delegates to internal draw helpers. |

## Invariants

1. `app.visible_rows` is recalculated on every `draw` call as `terminal_height - 3` (header + status bar + one implicit line), clamped to zero via saturating subtraction.
2. Byte coloring priority is strict: cursor style takes precedence over search-hit style, which takes precedence over modification/category color. This order is enforced identically in both the hex column and the ASCII column.
3. The cursor in the active editing panel (hex or ASCII) renders with a green background and bold modifier; the cursor in the non-active panel renders with a white background and no bold.
4. A hex byte group separator (double-space) is inserted after column index 7; all other inter-byte separators are single-space.
5. Non-printable bytes (outside ASCII graphic range and not space) are displayed as `'.'` in the ASCII column.
6. The offset column is always 8 hex digits wide, zero-padded.
7. Rows beyond `app.buffer.len()` are not rendered; the loop breaks early when `row_offset >= buffer length`.
8. The header displays `[+]` if and only if `app.buffer.is_dirty()` returns true.
9. The status bar input area shows `:{command_input}` in `Command` mode, `/{search_input}` in `Search` mode, and `status_message` (or empty) otherwise.

## Behavioral Examples

**Given** a buffer containing 32 bytes and `bytes_per_row` is 16
**When** `draw` is called with a terminal height of 10
**Then** `visible_rows` is set to 7, the header shows the filename and `32 bytes (0x20)`, two data rows are rendered (offsets `00000000` and `00000010`), and the status bar shows the current mode and cursor position.

---

**Given** byte at the cursor position has been modified
**When** the cursor is on that byte and mode is `EditHex`
**Then** the hex column shows the byte with black foreground on green background with bold; the ASCII column shows it with black foreground on white background (non-bold). The modification red color is suppressed by the cursor highlight.

---

**Given** a search has been performed with results at offsets 4 and 20
**When** the cursor is at offset 4
**Then** offset 4 renders with cursor styling (not search styling), and offset 20 renders with black-on-yellow search-hit styling in both hex and ASCII columns.

---

**Given** the buffer has no unsaved changes
**When** `draw` renders the header
**Then** the header displays the filename without the `[+]` dirty marker.

---

**Given** the mode is `Command` and `command_input` is `"w"`
**When** the status bar is drawn
**Then** the status bar shows a magenta mode label and the text `:w` in the input area.

## Error Cases

| Condition | Behavior |
|-----------|----------|
| File path has no filename component | Header displays `"???"` as the filename. |
| `app.buffer.get(offset)` returns `None` (offset beyond buffer) | Hex column emits three spaces (`"   "`); ASCII column emits a single space. No panic. |
| Terminal height is 3 or fewer rows | `visible_rows` saturates to 0; no hex-view rows are rendered; header and status bar still appear. |
| `status_message` is `None` in Normal/Edit modes | Status bar input area renders as empty (blank padding). |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `crate::app::App` | Entire application state: buffer, cursor, mode, scroll offset, search results, command/search input, status message. Mutated to update `visible_rows`. |
| `crate::app::Mode` | Enum (`Normal`, `Visual`, `EditHex`, `EditAscii`, `Command`, `Search`, `Inspector`, `InspectorEdit`) used for cursor styling and status-bar mode display. |
| `crate::inspector` | `interpret` function for generating inspector panel field data. |
| `ratatui::prelude::*` | Core TUI types: `Frame`, `Layout`, `Direction`, `Constraint`, `Rect`, `Style`, `Color`, `Modifier`, `Span`, `Line`. |
| `ratatui::widgets::{Block, Borders, Paragraph}` | Widgets used to compose the header, hex view container, and status bar. |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-29 | Initial spec |
| 2026-03-30 | Add inspector panel rendering, Inspector/InspectorEdit mode support in status bar |
