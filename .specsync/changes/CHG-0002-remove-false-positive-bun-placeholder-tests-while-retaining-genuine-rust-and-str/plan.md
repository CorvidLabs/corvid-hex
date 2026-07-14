---
change: CHG-0002-remove-false-positive-bun-placeholder-tests-while-retaining-genuine-rust-and-str
artifact: plan
---

# Plan

1. Remove `placeholder.test.ts`, `validate.test.ts`, `tests/search.test.ts`, and `tests/strings.test.ts` because they are stubs or duplicate implementation-independent assertions.
2. Remove the root placeholder file from TypeScript discovery.
3. Preserve the genuine structural Bun suites and all native Rust suites.
4. Run Clippy with warnings denied, the full Rust test suite, the mmap integration suite, the genuine Bun suite, strict SpecSync, and Trust health checks.
