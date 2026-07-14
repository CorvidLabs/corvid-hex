## ADDED
### REQUIREMENT REQ-input-001
Input handling SHALL dispatch every key event to the correct mode-specific handler.

Acceptance Criteria
- The same key can produce the documented behavior for each active mode without cross-mode dispatch.

### REQUIREMENT REQ-input-002
Normal mode SHALL support Vim-style `h`, `j`, `k`, `l`, `g`, `G`, `Ctrl-d`, and `Ctrl-u` navigation.

Acceptance Criteria
- Each listed key moves the cursor or viewport according to its documented Normal-mode behavior.

### REQUIREMENT REQ-input-003
Hexadecimal and ASCII edit modes SHALL correctly handle character input and cursor movement.

Acceptance Criteria
- Valid input changes the targeted byte and movement resets or advances edit state as documented.

### REQUIREMENT REQ-input-004
Command and search modes SHALL handle text input, backspace, and enter.

Acceptance Criteria
- Text can be entered, erased, and submitted in both modes.

### REQUIREMENT REQ-input-005
Mouse clicks SHALL translate to cursor-position changes.

Acceptance Criteria
- Clicking a rendered byte selects its corresponding buffer offset.

### REQUIREMENT REQ-input-006
Mouse-wheel input SHALL translate to vertical scrolling.

Acceptance Criteria
- Up and down wheel events move the visible byte rows in the corresponding direction.

