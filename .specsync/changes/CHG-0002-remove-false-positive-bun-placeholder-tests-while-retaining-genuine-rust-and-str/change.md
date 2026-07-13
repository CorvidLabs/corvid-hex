---
id: CHG-0002-remove-false-positive-bun-placeholder-tests-while-retaining-genuine-rust-and-str
state: accepted
type: refactor
base_commit: fe4be8367921c769a093e170d68cad9cc4857347
---

# Remove false-positive Bun placeholder tests while retaining genuine Rust and structural verification

## Intent

Remove false-positive Bun placeholder tests while retaining genuine Rust and structural verification

## Affected Canonical Specs

- None

## Acceptance Criteria

- All always-green Bun placeholder and stub tests are removed; TypeScript discovery includes only genuine tests; cargo clippy passes with warnings denied; 273 Rust unit tests, 4 mmap integration tests, and 40 genuine Bun tests pass; strict SpecSync and Trust remain healthy.

## No-spec Rationale

The cleanup removes always-green test stubs and narrows TypeScript test discovery without changing Corvid Hex runtime behavior or canonical requirement semantics.
