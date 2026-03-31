---
spec: diff.spec.md
---

## User Stories

- As a user, I want to compare two binary files byte-by-byte so that I can identify exactly where they differ
- As a user, I want to navigate between differences so that I can quickly jump to the next or previous change

## Acceptance Criteria

- Both files are loaded and compared byte-by-byte, producing a sorted list of differing offsets
- Navigation between diffs supports forward and backward traversal with wraparound
- Diff count and current position are tracked for display in the UI
- Files of different lengths correctly report differences in the trailing region

## Constraints

- Both files must fit in memory (no streaming diff)

## Out of Scope

- Structural or semantic diffing (e.g., ELF section comparison)
- Three-way merge
