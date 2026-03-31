---
spec: inspector.spec.md
---

## Key Decisions

- Read-only display types (binary, octal, ASCII, UTF-8) are not editable — they serve as reference views.
- Unsigned integer parsing supports `0x` hex prefix for convenience.
- UTF-8 decoding tries longest valid sequence first (up to 4 bytes) to correctly display multi-byte characters.

## Files to Read First

- `src/inspector.rs` — the entire module

## Current Status

- All field types implemented and tested
- Parsing supports decimal and hex input for unsigned types

## Notes

- The inspector panel UI is rendered by `render.rs` (`draw_inspector`), input handling is in `input.rs` (`handle_inspector`, `handle_inspector_edit`).
