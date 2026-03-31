---
module: diff
version: 1
status: draft
files:
  - src/diff.rs
db_tables: []
depends_on: []
---

## Purpose

Provides byte-by-byte binary diff comparison between two files. Loads both files into memory, computes a sorted list of offsets where they differ (including extra bytes when files have different lengths), and supports cursor-based navigation between differences with wraparound. Includes an XOR view toggle and summary statistics.

## Public API

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `DiffState` | `pub struct DiffState` | Core state for a binary diff session. Public fields: `left_data: Vec<u8>`, `right_data: Vec<u8>`, `left_name: String`, `right_name: String`, `diff_offsets: Vec<usize>`, `diff_index: usize`, `cursor: usize`, `scroll_offset: usize`, `bytes_per_row: usize`, `visible_rows: usize`, `status_message: Option<String>`, `xor_view: bool`. |
| `DiffStats` | `pub struct DiffStats` | Summary statistics for a diff. Fields: `total_bytes: usize`, `diff_count: usize`, `match_percentage: f64`, `first_diff: Option<usize>`, `left_size: usize`, `right_size: usize`. |
| `DiffState::open` | `pub fn open(left_path: &str, right_path: &str) -> anyhow::Result<Self>` | Opens two files, reads them into memory, computes diff offsets, and returns an initialized `DiffState` with default cursor/scroll/view settings. |
| `DiffState::max_len` | `pub fn max_len(&self) -> usize` | Returns the maximum length across both files. |
| `DiffState::stats` | `pub fn stats(&self) -> DiffStats` | Computes and returns summary statistics including match percentage, diff count, and first diff offset. |
| `DiffState::cursor_row` | `pub fn cursor_row(&self) -> usize` | Returns the row index for the current cursor position (`cursor / bytes_per_row`). |
| `DiffState::ensure_cursor_visible` | `pub fn ensure_cursor_visible(&mut self)` | Adjusts `scroll_offset` so the cursor's row is within the visible window. |
| `DiffState::move_cursor` | `pub fn move_cursor(&mut self, offset: isize)` | Moves cursor by a signed offset, clamped to `[0, max_len - 1]`. Calls `ensure_cursor_visible`. |
| `DiffState::move_cursor_to` | `pub fn move_cursor_to(&mut self, pos: usize)` | Sets cursor to an absolute position, clamped to `[0, max_len - 1]`. Calls `ensure_cursor_visible`. |
| `DiffState::page_down` | `pub fn page_down(&mut self)` | Moves cursor forward by `visible_rows * bytes_per_row` bytes. |
| `DiffState::page_up` | `pub fn page_up(&mut self)` | Moves cursor backward by `visible_rows * bytes_per_row` bytes. |
| `DiffState::next_diff` | `pub fn next_diff(&mut self)` | Jumps cursor to the next difference after the current position. Wraps to the first difference if at the end. Sets `status_message` with diff index info. |
| `DiffState::prev_diff` | `pub fn prev_diff(&mut self)` | Jumps cursor to the previous difference before the current position. Wraps to the last difference if at the beginning. Sets `status_message` with diff index info. |
| `DiffState::toggle_xor_view` | `pub fn toggle_xor_view(&mut self)` | Toggles `xor_view` and sets a status message indicating the new state. |
| `DiffState::left_byte` | `pub fn left_byte(&self, offset: usize) -> Option<u8>` | Returns the byte at the given offset in the left file, or `None` if beyond its length. |
| `DiffState::right_byte` | `pub fn right_byte(&self, offset: usize) -> Option<u8>` | Returns the byte at the given offset in the right file, or `None` if beyond its length. |
| `DiffState::is_diff` | `pub fn is_diff(&self, offset: usize) -> bool` | Returns true if the given offset is in the diff offsets list (binary search). |

## Invariants

1. `diff_offsets` is always sorted in ascending order and contains no duplicates.
2. `diff_offsets` includes offsets where one file has data and the other does not (extra bytes from the longer file).
3. `max_len()` returns the length of the longer file. For two empty files, it returns 0.
4. `cursor` is always clamped to `[0, max_len - 1]` after any movement operation. For empty files, cursor stays at 0.
5. `next_diff` and `prev_diff` wrap around when reaching the end/beginning of the diff list.
6. `stats().match_percentage` is 100.0 when both files are empty or identical, and 0.0 when all bytes differ.
7. `ensure_cursor_visible` guarantees `scroll_offset <= cursor_row() < scroll_offset + visible_rows` after any cursor movement.

## Behavioral Examples

**Identical files produce no diffs**
- Given: two files with identical content `"ABCDEF"`
- When: `DiffState::open` is called
- Then: `diff_offsets` is empty, `stats().match_percentage` is 100.0, `stats().first_diff` is `None`

**Different-length files mark extra bytes as diffs**
- Given: left file is `"AB"` (2 bytes), right file is `"ABCD"` (4 bytes)
- When: `DiffState::open` is called
- Then: `diff_offsets` is `[2, 3]`, `max_len()` is 4, `stats().left_size` is 2, `stats().right_size` is 4

**Navigation wraps around**
- Given: diffs at offsets `[2, 5]`, cursor at offset 5
- When: `next_diff()` is called
- Then: cursor wraps to offset 2, status message contains "Wrapped"

**XOR view toggle**
- Given: `xor_view` is false
- When: `toggle_xor_view()` is called
- Then: `xor_view` is true, status message is "XOR view enabled"

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Left file does not exist | `open` returns `Err` with message including the file path. |
| Right file does not exist | `open` returns `Err` with message including the file path. |
| Both files are empty | `max_len()` is 0, `diff_offsets` is empty, cursor stays at 0. Navigation methods are no-ops. |
| `next_diff`/`prev_diff` called with no differences | Sets `status_message` to "No differences" and does not move the cursor. |
| Cursor movement beyond bounds | Clamped to `[0, max_len - 1]` via `isize::clamp`. |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `std::fs` | Reading file contents into memory. |
| `std::path::Path` | Extracting file names from paths. |
| `anyhow` | Error type for `open` return value. |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-30 | Initial spec for binary diff mode |
