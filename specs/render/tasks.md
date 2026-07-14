---
spec: render.spec.md
---

## Tasks

- [x] Hex grid layout with offset, hex bytes, ASCII columns
- [x] Cursor highlighting in hex and ASCII panes
- [x] Status bar (mode, file name, offset, dirty indicator)
- [x] Command/search input line
- [x] Visual selection highlighting
- [x] Search result highlighting

## Gaps

- None identified in the current canonical behavior and native test suite.

## Review Sign-offs

- **Product evidence**: this migration preserves the documented product behavior.
- **QA evidence**: the repository-native Rust and Bun suites verify the existing behavior.
- **Design evidence**: no visual or interaction design changes are introduced.
- **Development evidence**: strict SpecSync, Clippy, and Trust gates verify the implementation boundary.
