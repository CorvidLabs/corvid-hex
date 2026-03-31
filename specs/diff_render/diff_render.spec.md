---
module: diff_render
version: 1
status: draft
files:
  - src/diff_render.rs
db_tables: []
depends_on:
  - specs/diff/diff.spec.md
---

## Purpose

Renders the terminal UI for binary diff mode. Draws a split-pane view with the left file on the left and the right file (or XOR view) on the right, separated by a vertical line. Includes a header showing file names and sizes, a stats bar with diff count and match percentage, and a status bar with cursor position and mode indicator.

## Public API

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `draw_diff` | `pub fn draw_diff(f: &mut Frame, state: &mut DiffState)` | Entry point for rendering a diff frame. Splits the terminal into header, stats bar, split hex view, and status bar. Updates `state.visible_rows` based on terminal height. |

## Invariants

1. `state.visible_rows` is recalculated on every `draw_diff` call as `terminal_height - 4` (header + stats + status bar + one implicit line), clamped to zero via saturating subtraction.
2. The split view divides the terminal width in half, with a `│` separator between panels.
3. `bytes_per_row` is auto-fitted to the available panel width using the formula `(half_width - 10) / 3`, clamped to at least 1.
4. Diff bytes are highlighted in red with a dark red background. Cursor position uses black-on-white styling.
5. Bytes that exist in one file but not the other (additions) are highlighted in green with bold.
6. In XOR view, the right panel shows `left ^ right` for each byte, with `0xFF` for offsets where only one file has data.
7. The offset column is always 8 hex digits wide, zero-padded.
8. Rows beyond `state.max_len()` are not rendered; the loop breaks early.
9. The stats bar shows diff count, match percentage, first diff offset (if any), and an `[XOR]` indicator when XOR view is active.
10. The status bar shows `DIFF-XOR` mode label when XOR view is active, `DIFF` otherwise.

## Behavioral Examples

**Normal diff view**
- Given: two files with 3 differences, cursor at offset 0
- When: `draw_diff` is called
- Then: header shows both filenames and sizes, stats bar shows "3 differences", left panel shows left file bytes, right panel shows right file bytes, differing bytes are red

**XOR view enabled**
- Given: `state.xor_view` is true, left byte is `0x41`, right byte is `0x42`
- When: the right panel renders offset
- Then: right panel shows `0x03` (XOR result) instead of `0x42`

**Auto-fit bytes per row**
- Given: terminal width is 80 (40 per panel)
- When: `draw_diff` is called
- Then: `bytes_per_row` is set to `min(state.bytes_per_row, (40 - 10) / 3)` = min(16, 10) = 10

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Terminal width is very small (< 22 columns) | `bytes_per_row` clamps to 1; layout degrades but does not panic. |
| Terminal height is 4 or fewer rows | `visible_rows` saturates to 0; no hex rows rendered; header, stats, and status bars still appear. |
| Offset beyond both files' lengths | Row loop breaks early; no out-of-bounds access. |
| Right panel width is 0 | Right panel is not rendered (guarded by `if right_width > 0`). |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `crate::diff::DiffState` | Entire diff state: file data, cursor, scroll offset, diff offsets, XOR view flag. Mutated to update `visible_rows` and `bytes_per_row`. |
| `ratatui::prelude::*` | Core TUI types: `Frame`, `Layout`, `Direction`, `Constraint`, `Rect`, `Style`, `Color`, `Modifier`, `Span`, `Line`. |
| `ratatui::widgets::Paragraph` | Widget used for rendering text in all UI sections. |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-30 | Initial spec for diff rendering |
