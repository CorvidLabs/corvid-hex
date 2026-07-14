---
spec: diff.spec.md
---

## Tasks

- [x] Load two files and compare byte-by-byte
- [x] Produce sorted list of differing offsets
- [x] Next/previous diff navigation with wraparound
- [x] Handle files of different lengths

## Gaps

- None identified in the current canonical behavior and native test suite.

## Review Sign-offs

- **Product evidence**: this migration preserves the documented product behavior.
- **QA evidence**: the repository-native Rust and Bun suites verify the existing behavior.
- **Design evidence**: no visual or interaction design changes are introduced.
- **Development evidence**: strict SpecSync, Clippy, and Trust gates verify the implementation boundary.
