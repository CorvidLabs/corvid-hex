---
spec: diff.spec.md
---

## Verification

- cargo test diff::tests verifies byte comparison, differing lengths, offsets, navigation, and diff predicates.
- bun test tests/binary-diff.test.ts verifies the diff module and binary fixture structure.
