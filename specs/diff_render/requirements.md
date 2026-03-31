---
spec: diff_render.spec.md
---

## User Stories

- As a user, I want a side-by-side diff view so that I can visually compare two binary files
- As a user, I want differing bytes highlighted so that changes are immediately visible
- As a user, I want an XOR view option so that I can see the bitwise difference between files

## Acceptance Criteria

- Split-pane layout renders left file on left and right file (or XOR) on right
- Vertical separator line clearly divides the two panes
- Header bar shows filenames and diff statistics
- Differing bytes are color-highlighted in both hex and ASCII columns
- Status bar shows navigation position and diff count

## Constraints

- Must render within a single terminal frame without flicker
- Layout must adapt to terminal width

## Out of Scope

- Resizable pane ratios
- Inline patch editing from diff view
