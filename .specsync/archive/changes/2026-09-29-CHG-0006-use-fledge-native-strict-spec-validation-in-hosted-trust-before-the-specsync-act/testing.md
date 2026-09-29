---
change: CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act
artifact: testing
---

# Testing

Local verification covers released and portable strict SpecSync checks at 100 percent coverage, all four
agent integrations, `fledge trust doctor`, the complete Trust-managed Fledge lifecycle, Clippy with
warnings denied, Rust tests, and Bun tests.

After push, both the normal CI workflow and unified Trust workflow must succeed at the exact commit. The
Trust log must show the Fledge-native strict spec task running before native checks and the later pinned
SpecSync 5.0.1 contract succeeding. Hosted success is not recorded until those runs complete.
