---
change: CHG-0007-complete-canonical-test-and-task-companions-required-by-the-hosted-fledge-strict
artifact: design
---

# Design

Add one concise testing companion per canonical module. Each companion names existing Rust or integration
tests and the real command that executes them. Add a format task companion that records the already
implemented detection, parsing, and rendering scope. Do not invent test results, requirements, or product
behavior.
