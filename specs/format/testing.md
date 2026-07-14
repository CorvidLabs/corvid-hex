---
spec: format.spec.md
---

## Verification

- cargo test format::tests verifies magic-byte detection, template parsing, field maps, and supported field types.
- bun test tests/format.test.ts verifies documented templates, fixtures, and registered built-in formats.
