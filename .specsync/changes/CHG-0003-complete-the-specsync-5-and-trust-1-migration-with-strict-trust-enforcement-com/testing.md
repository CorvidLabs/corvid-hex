---
change: CHG-0003-complete-the-specsync-5-and-trust-1-migration-with-strict-trust-enforcement-com
artifact: testing
---

# Testing

Run released SpecSync 5.0.1 in normal and forced strict modes at 100 percent, confirm all four agent integrations, run `fledge trust doctor`, and execute the full Fledge verify lane: Clippy with warnings denied, the complete Rust suite, and genuine Bun structural/regression tests. After pushing, require every check on the new commit to pass.

## Requirement Evidence

- `REQ-buffer-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-buffer-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-buffer-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-buffer-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-buffer-005`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-chx-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-chx-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-chx-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-chx-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-chx-005`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-diff-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-diff-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-diff-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-diff-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-diff-render-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-diff-render-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-diff-render-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-diff-render-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-diff-render-005`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-entropy-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-entropy-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-entropy-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-entropy-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-format-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-format-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-format-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-format-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-format-005`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-format-006`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-input-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-input-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-input-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-input-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-input-005`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-input-006`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-inspector-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-inspector-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-inspector-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-inspector-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-inspector-005`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-render-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-render-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-render-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-render-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-render-005`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-render-006`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-render-007`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-search-001`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-search-002`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-search-003`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-search-004`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
- `REQ-search-005`: covered by `cargo test`, `cargo clippy -- -D warnings`, and the genuine Bun structural/regression suite where applicable.
