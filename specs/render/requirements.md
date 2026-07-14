---
spec: render.spec.md
---

## User Stories

- As a user, I want a clear hex/ASCII split view so that I can read binary data efficiently
- As a user, I want the current byte highlighted and the view to scroll with my cursor so that I never lose my position
- As a user, I want a status bar showing mode, offset, and file info so that I always know where I am

## Acceptance Criteria

Stable, testable requirements are listed by ID after the scope sections.

## Constraints

- Must render full frame within a single terminal flush to avoid flicker
- Layout must adapt to terminal resize events

## Out of Scope

- Theming or color customization
- Split-pane views (diff_render handles that)

### REQ-render-001

The renderer SHALL provide a full-screen layout with a header, hex/ASCII view, optional inspector panel, and status bar.

Acceptance Criteria
- The enabled regions occupy their documented positions in a rendered frame.

### REQ-render-002

Each hex-view row SHALL show an offset, hexadecimal bytes, and an ASCII representation.

Acceptance Criteria
- A visible buffer row renders all three representations for the same offsets.

### REQ-render-003

The renderer SHALL highlight the cursor position in both hexadecimal and ASCII columns.

Acceptance Criteria
- The same selected offset is visibly highlighted in both columns.

### REQ-render-004

The view SHALL scroll as needed to keep the cursor visible.

Acceptance Criteria
- Moving beyond the current viewport updates the scroll offset to reveal the cursor.

### REQ-render-005

The renderer SHALL show visual selections with distinct highlighting.

Acceptance Criteria
- Every byte in an active visual selection receives selection styling.

### REQ-render-006

The renderer SHALL visually distinguish modified bytes from unmodified bytes.

Acceptance Criteria
- An edited byte uses modified styling while unchanged neighbors do not.

### REQ-render-007

The status bar SHALL show the current mode, cursor offset, file size, and dirty state.

Acceptance Criteria
- Each listed value matches current application state in a rendered frame.
