---
change: CHG-0007-complete-canonical-test-and-task-companions-required-by-the-hosted-fledge-strict
artifact: context
---

# Context

The hosted Fledge 1.7.0 structural fallback validates the complete companion set before the pinned
SpecSync action is available. Ten draft canonical modules lacked truthful testing companions, and the
format module also lacked its task companion. This made strict hosted validation fail with eleven
warnings despite the underlying Rust and Bun suites passing.
