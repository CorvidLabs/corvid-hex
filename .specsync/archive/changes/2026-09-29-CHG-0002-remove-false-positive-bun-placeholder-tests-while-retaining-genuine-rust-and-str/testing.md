---
change: CHG-0002-remove-false-positive-bun-placeholder-tests-while-retaining-genuine-rust-and-str
artifact: testing
---

# Testing

Required verification is configured in `.specsync/sdd.json` and runs:

- `cargo clippy -- -D warnings`
- `cargo test`, including 273 unit tests and 4 mmap integration tests
- `bun test`, containing 40 genuine structural tests after stub removal

The lifecycle also requires `specsync check --strict --force`, all four agent integrations, and a healthy Trust doctor result. Hosted CI is intentionally not claimed by this local change record.
