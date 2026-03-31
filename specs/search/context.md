---
spec: search.spec.md
---

## Key Decisions

- Pattern parsing: `x/` or `0x` prefix triggers hex mode; otherwise ASCII. Hex patterns require even digit count after stripping whitespace.
- Case-insensitive flag: `/i` suffix enables `eq_ignore_ascii_case` for ASCII patterns only (not hex).
- Incremental search: separate function updates results without clearing input or moving cursor — used for live highlighting as user types.
- Pattern length stored in `app.search_pattern_len` so render can highlight the full match span.
- Wrapping navigation: `next_search_result` and `prev_search_result` use modulo arithmetic to wrap around.
- Find-and-replace constrained to same-length patterns (overwrite mode only).

## Files to Read First

- `src/search.rs` — all search logic (pattern parsing, execution, navigation, find-replace)

## Current Status

Complete. ASCII/hex pattern parsing, case-insensitive flag, incremental search, full-span highlighting, wrapping navigation, and find-and-replace all implemented and tested.

## Notes

- Search results are stored as a sorted `Vec<usize>` of offsets. Binary search used for navigation.
