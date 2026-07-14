## ADDED
### REQUIREMENT REQ-diff-001
The diff engine SHALL compare both loaded files byte by byte and produce a sorted list of differing offsets.

Acceptance Criteria
- Returned offsets are ascending and contain every unequal byte position.

### REQUIREMENT REQ-diff-002
Diff navigation SHALL support forward and backward traversal with wraparound.

Acceptance Criteria
- Next from the last difference and previous from the first difference wrap to the opposite end.

### REQUIREMENT REQ-diff-003
The diff state SHALL track the total difference count and current position for UI display.

Acceptance Criteria
- The displayed count and position match the active difference list and selection.

### REQUIREMENT REQ-diff-004
The diff engine SHALL report trailing-region differences when the files have different lengths.

Acceptance Criteria
- Every offset present only in the longer file is reported as different.

