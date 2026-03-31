---
spec: entropy.spec.md
---

## User Stories

- As a user, I want to see an entropy heatmap so that I can identify compressed, encrypted, or structured regions in a binary file
- As a user, I want entropy calculated per-window so that the visualization has meaningful granularity

## Acceptance Criteria

- Shannon entropy is computed per configurable window size across the file
- Entropy values are mapped to a color gradient (low entropy = cool, high = warm)
- Average entropy can be computed over arbitrary byte ranges for the inspector
- Heatmap renders as a visual bar alongside the hex view

## Constraints

- Entropy calculation must be fast enough to not block the UI on large files

## Out of Scope

- Alternative entropy algorithms (e.g., min-entropy, Renyi entropy)
- Per-byte entropy (window-based only)
