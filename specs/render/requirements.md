---
spec: render.spec.md
---

## User Stories

- As a user, I want a clear hex/ASCII split view so that I can read binary data efficiently
- As a user, I want the current byte highlighted and the view to scroll with my cursor so that I never lose my position
- As a user, I want a status bar showing mode, offset, and file info so that I always know where I am

## Acceptance Criteria

- Full-screen layout with header, hex/ASCII view, optional inspector panel, and status bar
- Hex view shows offset column, hex bytes, and ASCII representation per row
- Cursor position is highlighted in both hex and ASCII columns
- View scrolls to keep cursor visible at all times
- Visual selection is rendered with distinct highlighting
- Modified bytes are visually distinct from unmodified bytes
- Status bar shows current mode, cursor offset, file size, and dirty state

## Constraints

- Must render full frame within a single terminal flush to avoid flicker
- Layout must adapt to terminal resize events

## Out of Scope

- Theming or color customization
- Split-pane views (diff_render handles that)
