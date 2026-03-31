---
spec: input.spec.md
---

## User Stories

- As a user, I want keyboard shortcuts to work consistently across modes so that I can build muscle memory
- As a user, I want mouse support for clicking on bytes and scrolling so that I can navigate intuitively

## Acceptance Criteria

- All key events are dispatched to the correct mode-specific handler
- Normal mode supports vim-style navigation (h/j/k/l, g/G, Ctrl-d/u)
- Edit modes (hex/ascii) correctly handle character input and cursor movement
- Command and search modes handle text input with backspace and enter
- Mouse clicks translate to cursor position changes
- Mouse scroll translates to vertical scrolling

## Constraints

- Input handling must be non-blocking to keep the event loop responsive
- Crossterm key event model is the source of truth for key representation

## Out of Scope

- User-configurable keybindings
- Macro recording and playback
