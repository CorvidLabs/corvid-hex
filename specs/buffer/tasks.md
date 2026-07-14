---
spec: buffer.spec.md
---

## Tasks

- [x] File open and memory-mapped read
- [x] Edit overlay (sparse HashMap of modified bytes)
- [x] get() reads overlay then original data
- [x] set() writes to overlay and marks dirty
- [x] save() flushes overlay to disk and clears dirty flag
- [x] Undo/redo stack (edit history with offset tracking)
- [x] is_dirty / is_empty / len accessors

## Gaps

- None identified in the current canonical behavior and native test suite.

## Review Sign-offs

- **Product evidence**: this migration preserves the documented product behavior.
- **QA evidence**: the repository-native Rust and Bun suites verify the existing behavior.
- **Design evidence**: no visual or interaction design changes are introduced.
- **Development evidence**: strict SpecSync, Clippy, and Trust gates verify the implementation boundary.
