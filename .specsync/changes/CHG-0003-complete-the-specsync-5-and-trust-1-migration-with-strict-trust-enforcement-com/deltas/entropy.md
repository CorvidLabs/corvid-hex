## ADDED
### REQUIREMENT REQ-entropy-001
The entropy engine SHALL compute Shannon entropy across the file using a configurable window size.

Acceptance Criteria
- Changing the window size changes the evaluated byte ranges while preserving valid Shannon values.

### REQUIREMENT REQ-entropy-002
The heatmap SHALL map low entropy to cool colors and high entropy to warm colors.

Acceptance Criteria
- Lower entropy values select cooler gradient entries than higher values.

### REQUIREMENT REQ-entropy-003
The entropy engine SHALL compute average entropy over arbitrary byte ranges for the inspector.

Acceptance Criteria
- A requested valid byte range returns its average window entropy.

### REQUIREMENT REQ-entropy-004
The renderer SHALL show the entropy heatmap as a visual bar alongside the hex view.

Acceptance Criteria
- Enabling the heatmap renders a bar adjacent to the hex content.

