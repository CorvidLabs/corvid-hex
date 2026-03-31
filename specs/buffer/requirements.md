---
spec: buffer.spec.md
---

## User Stories

- As a user, I want to open binary files of any size so that I can inspect and edit them
- As a user, I want edits to be non-destructive until I explicitly save so that I don't accidentally corrupt files
- As a user, I want undo/redo support so that I can reverse mistakes while editing

## Acceptance Criteria

- Files under 100 MB are read fully into memory for fast random access
- Files over 100 MB use memory-mapped I/O to avoid excessive memory usage
- Edits are stored in a sparse overlay and only flushed on explicit save
- Undo/redo stack tracks all byte modifications with correct offset tracking
- `is_dirty` accurately reflects whether unsaved changes exist

## Constraints

- Must handle files up to several GB via mmap without loading into heap
- Save operation must atomically flush all overlay edits to disk

## Out of Scope

- Concurrent file access / file locking
- Network or remote file support
