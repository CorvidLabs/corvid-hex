---
spec: input.spec.md
---

## Tasks

- [x] Normal mode key dispatch (quit, mode switches, navigation)
- [x] Vi-style navigation (h/j/k/l, g/G, 0/$)
- [x] Half-page scroll (Ctrl-D/Ctrl-U)
- [x] EditHex mode (two-nibble byte writing, navigation resets nibble)
- [x] EditAscii mode (ASCII character writing)
- [x] Tab toggle between EditHex/EditAscii
- [x] Command mode (typing, Esc, Enter, Backspace-exits-on-empty)
- [x] Search mode (typing, Esc, Enter executes, Backspace-exits-on-empty)
- [x] Visual mode (enter with v, navigate to extend, yank with y, Esc cancels)
- [x] Paste in Normal mode (p)
- [x] Undo/redo (u, Ctrl-R)
- [x] Search navigation (n/N)
- [x] Bookmark two-key sequences (m+letter to set, '+letter to jump)
- [x] Bookmark cancel on invalid follow-up key

## Gaps

- None identified in the current canonical behavior and native test suite.

## Review Sign-offs

- **Product evidence**: this migration preserves the documented product behavior.
- **QA evidence**: the repository-native Rust and Bun suites verify the existing behavior.
- **Design evidence**: no visual or interaction design changes are introduced.
- **Development evidence**: strict SpecSync, Clippy, and Trust gates verify the implementation boundary.
