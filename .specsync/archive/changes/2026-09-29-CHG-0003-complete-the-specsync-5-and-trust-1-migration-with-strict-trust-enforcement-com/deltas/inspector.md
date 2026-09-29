## ADDED
### REQUIREMENT REQ-inspector-001
The inspector SHALL display `u8`, `i8`, binary, octal, ASCII, and UTF-8 interpretations of the byte at the cursor.

Acceptance Criteria
- A selected byte produces every listed single-byte representation.

### REQUIREMENT REQ-inspector-002
The inspector SHALL display `u16`, `i16`, `u32`, `i32`, `f32`, `u64`, `i64`, and `f64` in little- and big-endian forms.

Acceptance Criteria
- Sufficient selected bytes produce both endian interpretations for every listed type.

### REQUIREMENT REQ-inspector-003
The inspector SHALL gracefully handle insufficient bytes near the end of a file.

Acceptance Criteria
- Truncated multi-byte values do not panic and are reported as unavailable.

### REQUIREMENT REQ-inspector-004
The inspector panel SHALL update as the cursor moves.

Acceptance Criteria
- Moving to a different byte refreshes the displayed interpretations.

### REQUIREMENT REQ-inspector-005
Editable inspector fields SHALL allow modifying values and writing them back to the buffer.

Acceptance Criteria
- Submitting a valid editable value updates the corresponding bytes in the buffer.

