---
spec: entropy.spec.md
---

## User Stories

- As a user, I want to see an entropy heatmap so that I can identify compressed, encrypted, or structured regions in a binary file
- As a user, I want entropy calculated per-window so that the visualization has meaningful granularity

## Acceptance Criteria

Stable, testable requirements are listed by ID after the scope sections.

## Constraints

- Entropy calculation must be fast enough to not block the UI on large files

## Out of Scope

- Alternative entropy algorithms (e.g., min-entropy, Renyi entropy)
- Per-byte entropy (window-based only)

### REQ-entropy-001

The entropy engine SHALL compute Shannon entropy across the file using a configurable window size.

Acceptance Criteria
- Changing the window size changes the evaluated byte ranges while preserving valid Shannon values.

### REQ-entropy-002

The heatmap SHALL map low entropy to cool colors and high entropy to warm colors.

Acceptance Criteria
- Lower entropy values select cooler gradient entries than higher values.

### REQ-entropy-003

The entropy engine SHALL compute average entropy over arbitrary byte ranges for the inspector.

Acceptance Criteria
- A requested valid byte range returns its average window entropy.

### REQ-entropy-004

The renderer SHALL show the entropy heatmap as a visual bar alongside the hex view.

Acceptance Criteria
- Enabling the heatmap renders a bar adjacent to the hex content.
