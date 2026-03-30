---
module: search
version: 1
status: draft
files:
  - src/search.rs
db_tables: []
depends_on:
  - specs/buffer/buffer.spec.md
---

## Purpose

Provides search functionality for the hex editor — parsing search queries (ASCII or hex patterns), executing full-buffer searches, and navigating between results. Supports both ASCII string searches and hex byte pattern searches (prefixed with `x/` or `0x`).

## Public API

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `parse_search_pattern` | `pub fn parse_search_pattern(query: &str) -> Option<Vec<u8>>` | Parses a search query into a byte pattern. Hex patterns use `x/` or `0x` prefix; otherwise treated as ASCII. Returns `None` for empty or invalid input. |
| `execute_search` | `pub fn execute_search(app: &mut App)` | Reads `app.search_input`, parses it, finds all occurrences in the buffer, populates `app.search_results` and `app.search_pattern_len`, and jumps to the first match at or after the cursor. Supports `/i` suffix for case-insensitive ASCII matching. |
| `incremental_search` | `pub fn incremental_search(app: &mut App)` | Updates search results from `app.search_input` without clearing input or moving cursor. Used for live highlighting as the user types. |
| `next_search_result` | `pub fn next_search_result(app: &mut App)` | Advances to the next search result (wraps around). Updates cursor and status message. |
| `prev_search_result` | `pub fn prev_search_result(app: &mut App)` | Moves to the previous search result (wraps around). Updates cursor and status message. |
| `execute_replace` | `pub fn execute_replace(app: &mut App, find: &str, replace: &str)` | Replaces all occurrences of `find` with `replace` in the buffer. Both patterns are parsed via `parse_search_pattern`. Requires same-length patterns (overwrite mode). Sets status message with replacement count. |

## Invariants

1. `parse_search_pattern` returns `None` for empty/whitespace-only queries.
2. Hex patterns must have an even number of hex digits (after stripping whitespace); odd-length returns `None`.
3. `execute_search` clears `search_input` and `search_results` before populating new results. Sets `search_pattern_len` to the byte length of the parsed pattern.
4. After `execute_search`, `search_index` points to the first result at or after the current cursor, or wraps to index 0 if all results are before the cursor.
5. `incremental_search` updates `search_results` and `search_pattern_len` from `search_input` without clearing it or moving the cursor.
6. A `/i` suffix on the search query enables case-insensitive ASCII matching (only for ASCII patterns, not hex).
5. `next_search_result` and `prev_search_result` wrap around the result list cyclically.
6. All three navigation functions update `app.status_message` with match position info.

## Behavioral Examples

**ASCII search**
- Given: query is `"hello"`
- When: `parse_search_pattern("hello")` is called
- Then: returns `Some(vec![0x68, 0x65, 0x6C, 0x6C, 0x6F])`

**Hex search with x/ prefix**
- Given: query is `"x/DEADBEEF"`
- When: `parse_search_pattern("x/DEADBEEF")` is called
- Then: returns `Some(vec![0xDE, 0xAD, 0xBE, 0xEF])`

**Hex search with spaces**
- Given: query is `"x/DE AD BE EF"`
- When: `parse_search_pattern` is called
- Then: whitespace is stripped and returns `Some(vec![0xDE, 0xAD, 0xBE, 0xEF])`

**Odd-length hex pattern rejected**
- Given: query is `"x/DEA"`
- When: `parse_search_pattern` is called
- Then: returns `None`

**Search wraps with next/prev**
- Given: 3 search results exist and `search_index` is 2 (last)
- When: `next_search_result` is called
- Then: `search_index` wraps to 0

**No results**
- Given: search pattern does not match any bytes in buffer
- When: `execute_search` is called
- Then: `search_results` is empty and status message says "Pattern not found: {query}"

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Empty search query | `parse_search_pattern` returns `None`; `execute_search` sets status to "Invalid search pattern" |
| Invalid hex digits in pattern | `parse_search_pattern` returns `None` |
| Odd number of hex digits | `parse_search_pattern` returns `None` |
| No search results when calling next/prev | Functions return immediately (no-op) |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `crate::app::App` | Application state: `search_input`, `search_results`, `search_index`, `search_pattern_len`, `cursor`, `buffer`, `status_message` |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-29 | Initial spec |
| 2026-03-29 | Add execute_replace export |
| 2026-03-30 | Add incremental_search, case-insensitive /i flag, search_pattern_len for full-span highlighting |
