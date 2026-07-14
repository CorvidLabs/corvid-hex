---
spec: diff.spec.md
---

## User Stories

- As a user, I want to compare two binary files byte-by-byte so that I can identify exactly where they differ
- As a user, I want to navigate between differences so that I can quickly jump to the next or previous change

## Acceptance Criteria

Stable, testable requirements are listed by ID after the scope sections.

## Constraints

- Both files must fit in memory (no streaming diff)

## Out of Scope

- Structural or semantic diffing (e.g., ELF section comparison)
- Three-way merge

### REQ-diff-001

The diff engine SHALL compare both loaded files byte by byte and produce a sorted list of differing offsets.

Acceptance Criteria
- Returned offsets are ascending and contain every unequal byte position.

### REQ-diff-002

Diff navigation SHALL support forward and backward traversal with wraparound.

Acceptance Criteria
- Next from the last difference and previous from the first difference wrap to the opposite end.

### REQ-diff-003

The diff state SHALL track the total difference count and current position for UI display.

Acceptance Criteria
- The displayed count and position match the active difference list and selection.

### REQ-diff-004

The diff engine SHALL report trailing-region differences when the files have different lengths.

Acceptance Criteria
- Every offset present only in the longer file is reported as different.
