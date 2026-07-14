## ADDED
### REQUIREMENT REQ-render-001
The renderer SHALL provide a full-screen layout with a header, hex/ASCII view, optional inspector panel, and status bar.

Acceptance Criteria
- The enabled regions occupy their documented positions in a rendered frame.

### REQUIREMENT REQ-render-002
Each hex-view row SHALL show an offset, hexadecimal bytes, and an ASCII representation.

Acceptance Criteria
- A visible buffer row renders all three representations for the same offsets.

### REQUIREMENT REQ-render-003
The renderer SHALL highlight the cursor position in both hexadecimal and ASCII columns.

Acceptance Criteria
- The same selected offset is visibly highlighted in both columns.

### REQUIREMENT REQ-render-004
The view SHALL scroll as needed to keep the cursor visible.

Acceptance Criteria
- Moving beyond the current viewport updates the scroll offset to reveal the cursor.

### REQUIREMENT REQ-render-005
The renderer SHALL show visual selections with distinct highlighting.

Acceptance Criteria
- Every byte in an active visual selection receives selection styling.

### REQUIREMENT REQ-render-006
The renderer SHALL visually distinguish modified bytes from unmodified bytes.

Acceptance Criteria
- An edited byte uses modified styling while unchanged neighbors do not.

### REQUIREMENT REQ-render-007
The status bar SHALL show the current mode, cursor offset, file size, and dirty state.

Acceptance Criteria
- Each listed value matches current application state in a rendered frame.

