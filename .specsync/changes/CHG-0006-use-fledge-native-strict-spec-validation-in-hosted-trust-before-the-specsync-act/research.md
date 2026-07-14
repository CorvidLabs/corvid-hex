---
change: CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act
artifact: research
---

# Research

The exact-head hosted Trust log failed at the first lifecycle task with `specsync: not found`. The native
CI workflow passed at the same commit, isolating the failure to Trust's tool-installation order rather
than repository code or tests.

Fledge 1.7.0 exposes `fledge spec check --strict`, which is available as soon as Trust installs Fledge.
Strict mode blocks changed-source and companion drift. Trust's subsequent pinned SpecSync 5.0.1 contract
step reads the committed policy and independently enforces 100 percent coverage.
