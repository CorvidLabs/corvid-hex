---
spec: chx.spec.md
---

## User Stories

- As a user, I want a responsive terminal hex editor so that I can view and edit binary files from the command line
- As a user, I want modal editing (normal, visual, edit, command, search) so that I can efficiently navigate and modify data
- As a user, I want the editor to cleanly set up and tear down the terminal so that my shell is not corrupted on exit

## Acceptance Criteria

Stable, testable requirements are listed by ID after the scope sections.

## Constraints

- Single-threaded event loop — must remain responsive under large files
- Crossterm-based terminal handling for cross-platform support

## Out of Scope

- Multi-file / tabbed editing
- Plugin or scripting system

### REQ-chx-001

The application SHALL start in Normal mode with the file loaded and the hex view rendered.

Acceptance Criteria
- Launching with a valid file presents its bytes in Normal mode.

### REQ-chx-002

The application SHALL provide well-defined transitions that make every mode reachable from Normal mode.

Acceptance Criteria
- Normal, visual, hexadecimal edit, ASCII edit, command, and search modes are reachable through documented input.

### REQ-chx-003

Command mode SHALL support save, quit, save-and-quit, goto, and search commands.

Acceptance Criteria
- Each documented command is parsed and dispatches its corresponding editor operation.

### REQ-chx-004

The application SHALL restore the terminal to its original state after both a clean exit and a panic.

Acceptance Criteria
- Raw mode and alternate-screen state are cleared on normal and panic cleanup paths.

### REQ-chx-005

The event loop SHALL process keyboard and mouse events without blocking.

Acceptance Criteria
- Keyboard and mouse events continue to update the editor while the event loop is running.
