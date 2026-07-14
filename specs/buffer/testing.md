---
spec: buffer.spec.md
---

## Verification

- cargo test buffer::tests verifies in-memory and memory-mapped reads, edits, undo/redo, search, and save behavior.
- cargo test --test mmap_integration verifies large-file mapping, editing, search, and persisted saves.
