## ADDED
### REQUIREMENT REQ-diff-render-001
The diff renderer SHALL show the left file in the left pane and the right file or XOR view in the right pane.

Acceptance Criteria
- The selected right-side mode renders either source bytes or their XOR result without swapping panes.

### REQUIREMENT REQ-diff-render-002
The diff renderer SHALL clearly divide the two panes with a vertical separator line.

Acceptance Criteria
- A visible separator is rendered between left and right content.

### REQUIREMENT REQ-diff-render-003
The header bar SHALL show filenames and difference statistics.

Acceptance Criteria
- Both filenames and the current difference statistics appear in the header.

### REQUIREMENT REQ-diff-render-004
The diff renderer SHALL color-highlight differing bytes in both hexadecimal and ASCII columns.

Acceptance Criteria
- A differing offset receives difference styling in both representations.

### REQUIREMENT REQ-diff-render-005
The status bar SHALL show the navigation position and difference count.

Acceptance Criteria
- Navigating differences updates both values in the status bar.

