---
spec: buffer.spec.md
---

## Key Decisions

- Dual-backing strategy: files under 100 MB loaded as `Vec<u8>`, larger files use `memmap2::Mmap`. Abstracted via internal `Backing` enum.
- Sparse edit overlay (`HashMap<usize, u8>`) — edits never modify the backing store directly, enabling cheap undo and non-destructive editing.
- Self-cleaning edits: writing a byte that matches the original removes it from the overlay instead of storing a no-op.
- Undo/redo via dual stacks of `UndoEntry { offset, previous_value }` — symmetric push/pop between stacks.
- Chunked save for mmap files: reads 64 KB chunks, applies overlay edits, writes to temp file, then atomic rename.

## Files to Read First

- `src/buffer.rs` — entire module implementation (open, get/set, undo/redo, save, find)
- `src/lib.rs` — re-exports `Buffer`

## Current Status

Complete. All core functionality implemented and tested (30+ tests covering open, get/set, undo/redo, save, find, byte counting, edge cases).

## Notes

- `is_dirty` returns true if the overlay is non-empty, accurately reflecting unsaved state.
- Find operations scan the full buffer (overlay-aware) — no indexing.
