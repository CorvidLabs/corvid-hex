---
spec: diff.spec.md
---

## Key Decisions

- Pre-computed `diff_offsets: Vec<usize>` stores all differing byte indices at load time — binary-searched for O(log n) navigation.
- Files of different lengths supported: bytes beyond the shorter file treated as differences.
- Wrapping navigation: `next_diff()` and `prev_diff()` wrap at boundaries.
- Viewport management (scroll offset, visible rows, cursor) decoupled from rendering.
- Optional XOR view mode toggleable at runtime without recomputing diffs.

## Files to Read First

- `src/diff.rs` — `DiffState` struct, diff computation, navigation logic

## Current Status

Complete. Byte-by-byte comparison, wrapping navigation, XOR mode, and stats all implemented. Launched in v0.2.0.

## Notes

- Stats computed on-demand via `stats()` rather than cached.
- 18 tests covering edge cases: empty files, different sizes, navigation wrapping.
