---
change: CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act
artifact: plan
---

# Plan

1. Record the exact hosted failure and the action's tool-installation order.
2. Configure the existing lifecycle lane to use Fledge-native strict SpecSync validation.
3. Re-run the full local SpecSync, agent, Trust doctor, lifecycle, and native verification suite.
4. Push the correction and require exact-head hosted CI and Trust success before readiness or merge.
