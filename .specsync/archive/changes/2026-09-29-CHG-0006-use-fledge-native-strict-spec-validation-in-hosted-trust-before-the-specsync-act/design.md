---
change: CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act
artifact: design
---

# Design

Replace the `spec` task's bare executable invocation with `fledge spec check --strict`. Keep it as the
first step in the existing `verify` lane, followed by Clippy, Rust tests, and Bun tests.

No additional setup step or duplicate workflow is introduced. The Fledge lifecycle provides early strict
drift detection; the later pinned SpecSync contract remains the authoritative coverage check. Product
behavior, canonical requirements, and the progressive-provenance policy remain unchanged.
