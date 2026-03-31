---
spec: chx.spec.md
---

## User Stories

- As a user, I want a responsive terminal hex editor so that I can view and edit binary files from the command line
- As a user, I want modal editing (normal, visual, edit, command, search) so that I can efficiently navigate and modify data
- As a user, I want the editor to cleanly set up and tear down the terminal so that my shell is not corrupted on exit

## Acceptance Criteria

- Application starts in Normal mode with the file loaded and hex view rendered
- Mode transitions are well-defined and all modes are reachable from Normal mode
- Command mode supports save, quit, save-and-quit, goto, and search commands
- Terminal is restored to its original state on both clean exit and panic
- Event loop processes keyboard and mouse events without blocking

## Constraints

- Single-threaded event loop — must remain responsive under large files
- Crossterm-based terminal handling for cross-platform support

## Out of Scope

- Multi-file / tabbed editing
- Plugin or scripting system
