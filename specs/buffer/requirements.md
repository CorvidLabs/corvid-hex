---
spec: buffer.spec.md
---

## User Stories

- As a user, I want to open binary files of any size so that I can inspect and edit them
- As a user, I want edits to be non-destructive until I explicitly save so that I don't accidentally corrupt files
- As a user, I want undo/redo support so that I can reverse mistakes while editing

## Acceptance Criteria

Stable, testable requirements are listed by ID after the scope sections.

## Constraints

- Must handle files up to several GB via mmap without loading into heap
- Save operation must atomically flush all overlay edits to disk

## Out of Scope

- Concurrent file access / file locking
- Network or remote file support

### REQ-buffer-001

The buffer SHALL read files under 100 MB fully into memory for fast random access.

Acceptance Criteria
- Opening a file smaller than 100 MB selects the in-memory backing.

### REQ-buffer-002

The buffer SHALL use memory-mapped I/O for files over 100 MB to avoid excessive memory usage.

Acceptance Criteria
- Opening a file larger than 100 MB selects the memory-mapped backing.

### REQ-buffer-003

The buffer SHALL store edits in a sparse overlay and flush them only on explicit save.

Acceptance Criteria
- An edit changes the overlay without changing the persisted file until save is invoked.

### REQ-buffer-004

The buffer SHALL track all byte modifications in its undo and redo stacks with correct offsets.

Acceptance Criteria
- Undo and redo restore the expected byte at the original edit offset.

### REQ-buffer-005

`is_dirty` SHALL accurately report whether unsaved changes exist.

Acceptance Criteria
- `is_dirty` is true after an effective edit and false after the edits are reverted or saved.
