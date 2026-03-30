---
module: buffer
version: 1
status: draft
files:
  - src/buffer.rs
db_tables: []
depends_on: []
---

## Purpose

Provides a copy-on-open file buffer with a sparse edit overlay. Reads the entire file into memory on open, stores modifications in a `HashMap<usize, u8>` overlay, and merges edits on save. This allows efficient overwrite-mode editing without rewriting the full buffer on every keystroke.

## Public API

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `Buffer` | `pub struct Buffer` | Copy-on-open file buffer. Public field: `path: PathBuf`. Internal: `original: Vec<u8>`, `edits: HashMap<usize, u8>`, `undo_stack: Vec<UndoEntry>`, `redo_stack: Vec<UndoEntry>`. |
| `open` | `pub fn open(path: &str) -> Result<Self>` | Opens a file (or creates an empty buffer if path doesn't exist). Reads entire contents into memory. |
| `len` | `pub fn len(&self) -> usize` | Returns the length of the original data (not affected by edits). |
| `is_empty` | `pub fn is_empty(&self) -> bool` | Returns true if the original data is empty. |
| `get` | `pub fn get(&self, offset: usize) -> Option<u8>` | Returns the byte at offset, preferring the edit overlay. Returns `None` if offset >= len. |
| `set` | `pub fn set(&mut self, offset: usize, value: u8)` | Sets a byte in the edit overlay. If value matches original, removes the edit (restoring original). Pushes previous state onto the undo stack and clears the redo stack. |
| `undo` | `pub fn undo(&mut self) -> Option<usize>` | Undoes the last edit, restoring the previous byte value. Returns the offset that was changed, or `None` if the undo stack is empty. Pushes current state onto the redo stack. |
| `redo` | `pub fn redo(&mut self) -> Option<usize>` | Redoes the last undone edit. Returns the offset that was changed, or `None` if the redo stack is empty. Pushes current state onto the undo stack. |
| `is_modified` | `pub fn is_modified(&self, offset: usize) -> bool` | Returns true if the byte at offset has been edited. |
| `is_dirty` | `pub fn is_dirty(&self) -> bool` | Returns true if any edits exist in the overlay. |
| `save` | `pub fn save(&mut self) -> Result<()>` | Merges edits into original data, writes to disk, clears the edit overlay. |
| `find` | `pub fn find(&self, pattern: &[u8], start: usize) -> Option<usize>` | Searches for a byte pattern starting at `start`, respecting the edit overlay. Returns first match offset. |

## Invariants

1. `len()` always reflects the original file size — edits cannot change the buffer length (overwrite-only).
2. Setting a byte to its original value removes it from the edit overlay (self-cleaning).
3. After `save()`, `is_dirty()` returns false and all edits are merged into `original`.
4. `get()` always returns the edited value when present, falling back to original data.
5. `find()` reads through the edit overlay via `get()`, so searches reflect unsaved modifications.
6. Offsets beyond `len()` are silently ignored by `set()` and return `None` from `get()`.
7. Each `set()` call pushes the previous state onto the undo stack and clears the redo stack.
8. `undo()` and `redo()` move entries between the undo and redo stacks symmetrically.
9. After `save()`, undo/redo stacks are preserved — only the edit overlay is cleared.

## Behavioral Examples

**Opening a non-existent file**
- Given: the path does not exist on disk
- When: `Buffer::open("newfile.bin")` is called
- Then: returns a Buffer with empty original data and no edits

**Edit then revert**
- Given: original byte at offset 5 is `0x41`
- When: `set(5, 0xFF)` then `set(5, 0x41)` are called
- Then: `is_modified(5)` returns false and `is_dirty()` returns false

**Save merges edits**
- Given: buffer has 3 edits in the overlay
- When: `save()` is called successfully
- Then: edits are written to disk, `original` is updated, overlay is empty, `is_dirty()` is false

**Undo and redo**
- Given: original byte at offset 3 is `0x00`
- When: `set(3, 0xFF)` then `undo()` are called
- Then: `get(3)` returns `Some(0x00)`, `is_modified(3)` returns false, and `redo()` returns `Some(3)` restoring the edit

**New edit clears redo stack**
- Given: `set(0, 0xAA)` then `undo()` have been called (redo stack is non-empty)
- When: `set(0, 0xBB)` is called
- Then: the redo stack is cleared; `redo()` returns `None`

**Find across edit boundary**
- Given: original data is `[0x00, 0x00, 0x00]` and `set(1, 0xAB)` has been called
- When: `find(&[0x00, 0xAB], 0)` is called
- Then: returns `Some(0)`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| File exists but is unreadable | `open` returns `Err` with context "Failed to read {path}" |
| File path is not writable on save | `save` returns `Err` with context "Failed to write {path}" |
| `get` called with offset >= `len()` | Returns `None` |
| `set` called with offset >= `len()` | Silently ignored |
| `find` with empty pattern | Returns `None` |
| `find` on empty buffer | Returns `None` |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `anyhow` | `Result`, `Context` for error propagation |
| `std::collections::HashMap` | Sparse edit overlay storage |
| `std::fs` | `read` and `write` for file I/O |
| `std::path::{Path, PathBuf}` | File path handling |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-29 | Initial spec |
| 2026-03-29 | Add undo/redo to Public API, invariants, and behavioral examples |
