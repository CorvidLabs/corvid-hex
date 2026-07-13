---
change: CHG-0002-remove-false-positive-bun-placeholder-tests-while-retaining-genuine-rust-and-str
artifact: context
---

# Context

Four Bun files were test-discovery stubs or duplicated assertions while always passing independently of the Rust implementation. The repository already has comprehensive native Rust tests and genuine Bun structural tests. Keeping the stubs would inflate the apparent test count and permit a false-green signal.

This correction removes the four stubs and narrows `tsconfig.json` discovery to the remaining genuine test files. It does not change the `chx` binary, library behavior, public API, or canonical requirement semantics.
