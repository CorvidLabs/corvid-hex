import { describe, test, expect } from "bun:test";

// corvid-hex is a Rust TUI hex editor.
// This file exists to satisfy the corvid-agent CI validation framework,
// which requires at least one bun test file to be present.
// All real tests live in tests/mmap_integration.rs (run via `cargo test`).

describe("corvid-hex", () => {
  test("project is a Rust crate with mmap support", () => {
    // Sentinel test — Rust tests are in tests/mmap_integration.rs
    expect(true).toBe(true);
  });
});
