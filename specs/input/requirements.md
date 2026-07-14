---
spec: input.spec.md
---

## User Stories

- As a user, I want keyboard shortcuts to work consistently across modes so that I can build muscle memory
- As a user, I want mouse support for clicking on bytes and scrolling so that I can navigate intuitively

## Acceptance Criteria

Stable, testable requirements are listed by ID after the scope sections.

## Constraints

- Input handling must be non-blocking to keep the event loop responsive
- Crossterm key event model is the source of truth for key representation

## Out of Scope

- User-configurable keybindings
- Macro recording and playback

### REQ-input-001

Input handling SHALL dispatch every key event to the correct mode-specific handler.

Acceptance Criteria
- The same key can produce the documented behavior for each active mode without cross-mode dispatch.

### REQ-input-002

Normal mode SHALL support Vim-style `h`, `j`, `k`, `l`, `g`, `G`, `Ctrl-d`, and `Ctrl-u` navigation.

Acceptance Criteria
- Each listed key moves the cursor or viewport according to its documented Normal-mode behavior.

### REQ-input-003

Hexadecimal and ASCII edit modes SHALL correctly handle character input and cursor movement.

Acceptance Criteria
- Valid input changes the targeted byte and movement resets or advances edit state as documented.

### REQ-input-004

Command and search modes SHALL handle text input, backspace, and enter.

Acceptance Criteria
- Text can be entered, erased, and submitted in both modes.

### REQ-input-005

Mouse clicks SHALL translate to cursor-position changes.

Acceptance Criteria
- Clicking a rendered byte selects its corresponding buffer offset.

### REQ-input-006

Mouse-wheel input SHALL translate to vertical scrolling.

Acceptance Criteria
- Up and down wheel events move the visible byte rows in the corresponding direction.
