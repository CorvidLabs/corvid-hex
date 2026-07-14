---
spec: entropy.spec.md
---

## Tasks

- [x] Per-window Shannon entropy calculation
- [x] Entropy-to-color gradient mapping
- [x] Average entropy over arbitrary byte ranges
- [x] Heatmap bar rendering

## Gaps

- None identified in the current canonical behavior and native test suite.

## Review Sign-offs

- **Product evidence**: this migration preserves the documented product behavior.
- **QA evidence**: the repository-native Rust and Bun suites verify the existing behavior.
- **Design evidence**: no visual or interaction design changes are introduced.
- **Development evidence**: strict SpecSync, Clippy, and Trust gates verify the implementation boundary.
