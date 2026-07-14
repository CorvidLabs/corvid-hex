---
spec: search.spec.md
---

## Tasks

- [x] Parse search patterns (ASCII text, hex with 0x prefix, hex with x/ prefix)
- [x] Hex patterns with spaces (e.g. "0xFF 00 AB")
- [x] execute_search: find all matches, jump to first
- [x] next/prev search result cycling
- [x] execute_replace: find-and-replace with same-length constraint
- [x] Replace supports both ASCII and hex patterns
- [x] Case-insensitive search via `/i` suffix
- [x] Incremental search (live highlighting while typing)
- [x] Full-span match highlighting (all bytes in match, not just start byte)
- [x] search_pattern_len stored in App for render to use

## Gaps

- None identified in the current canonical behavior and native test suite.

## Review Sign-offs

- **Product evidence**: this migration preserves the documented product behavior.
- **QA evidence**: the repository-native Rust and Bun suites verify the existing behavior.
- **Design evidence**: no visual or interaction design changes are introduced.
- **Development evidence**: strict SpecSync, Clippy, and Trust gates verify the implementation boundary.
