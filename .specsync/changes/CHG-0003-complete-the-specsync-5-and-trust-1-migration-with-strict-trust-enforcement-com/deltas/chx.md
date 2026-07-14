## ADDED
### REQUIREMENT REQ-chx-001
The application SHALL start in Normal mode with the file loaded and the hex view rendered.

Acceptance Criteria
- Launching with a valid file presents its bytes in Normal mode.

### REQUIREMENT REQ-chx-002
The application SHALL provide well-defined transitions that make every mode reachable from Normal mode.

Acceptance Criteria
- Normal, visual, hexadecimal edit, ASCII edit, command, and search modes are reachable through documented input.

### REQUIREMENT REQ-chx-003
Command mode SHALL support save, quit, save-and-quit, goto, and search commands.

Acceptance Criteria
- Each documented command is parsed and dispatches its corresponding editor operation.

### REQUIREMENT REQ-chx-004
The application SHALL restore the terminal to its original state after both a clean exit and a panic.

Acceptance Criteria
- Raw mode and alternate-screen state are cleared on normal and panic cleanup paths.

### REQUIREMENT REQ-chx-005
The event loop SHALL process keyboard and mouse events without blocking.

Acceptance Criteria
- Keyboard and mouse events continue to update the editor while the event loop is running.

