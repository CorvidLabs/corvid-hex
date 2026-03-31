---
spec: input.spec.md
---

## Key Decisions

- Mode-based dispatch: `handle_key()` routes to mode-specific handlers (normal, edit_hex, edit_ascii, command, search, visual, strings, inspector, inspector_edit).
- Hex nibble buffering: EditHex stores partial nibble in `app.hex_nibble: Option<u8>`, writes on full byte (high+low). Navigation resets the nibble.
- Pending bookmark operations: two-key sequences (`m`+letter for set, `'`+letter for jump) handled via `app.pending_bookmark` field.
- Mouse support: clicks (positioning + mode exit), drags (visual selection), scroll wheel (3-row movement).
- Tab toggles between EditHex and EditAscii, resetting `hex_nibble` when entering EditHex.

## Files to Read First

- `src/input.rs` — all input handling (1000+ lines, mode dispatch is the entry point)

## Current Status

Complete. All modes implemented: normal, visual, edit (hex/ascii), command, search, strings, inspector, inspector_edit. Bookmarks, mouse, undo/redo, copy/paste all working.

## Notes

- Command mode (`:`) handles save, quit, goto, set bytes-per-row, and template commands.
- Search mode delegates to `search.rs` for pattern parsing and result navigation.
