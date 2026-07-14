---
spec: diff_render.spec.md
---

## User Stories

- As a user, I want a side-by-side diff view so that I can visually compare two binary files
- As a user, I want differing bytes highlighted so that changes are immediately visible
- As a user, I want an XOR view option so that I can see the bitwise difference between files

## Acceptance Criteria

Stable, testable requirements are listed by ID after the scope sections.

## Constraints

- Must render within a single terminal frame without flicker
- Layout must adapt to terminal width

## Out of Scope

- Resizable pane ratios
- Inline patch editing from diff view

### REQ-diff-render-001

The diff renderer SHALL show the left file in the left pane and the right file or XOR view in the right pane.

Acceptance Criteria
- The selected right-side mode renders either source bytes or their XOR result without swapping panes.

### REQ-diff-render-002

The diff renderer SHALL clearly divide the two panes with a vertical separator line.

Acceptance Criteria
- A visible separator is rendered between left and right content.

### REQ-diff-render-003

The header bar SHALL show filenames and difference statistics.

Acceptance Criteria
- Both filenames and the current difference statistics appear in the header.

### REQ-diff-render-004

The diff renderer SHALL color-highlight differing bytes in both hexadecimal and ASCII columns.

Acceptance Criteria
- A differing offset receives difference styling in both representations.

### REQ-diff-render-005

The status bar SHALL show the navigation position and difference count.

Acceptance Criteria
- Navigating differences updates both values in the status bar.
