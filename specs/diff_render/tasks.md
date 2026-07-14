---
spec: diff_render.spec.md
---

## Tasks

- [x] Split-pane layout with vertical separator
- [x] Header bar with filenames and diff stats
- [x] Color-highlighted differing bytes
- [x] XOR view mode
- [x] Status bar with navigation position

## Gaps

- None identified in the current canonical behavior and native test suite.

## Review Sign-offs

- **Product evidence**: this migration preserves the documented product behavior.
- **QA evidence**: the repository-native Rust and Bun suites verify the existing behavior.
- **Design evidence**: no visual or interaction design changes are introduced.
- **Development evidence**: strict SpecSync, Clippy, and Trust gates verify the implementation boundary.
