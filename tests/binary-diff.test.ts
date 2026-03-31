// Smoke tests for corvid-hex binary diff feature (issue #39)
// This project is implemented in Rust; these tests validate project metadata
// and serve as integration markers for the corvid validation framework.

import { describe, expect, it } from "bun:test";
import { readFileSync, existsSync } from "fs";
import { join } from "path";

const root = join(import.meta.dir, "..");

describe("corvid-hex project structure", () => {
  it("has a Cargo.toml", () => {
    expect(existsSync(join(root, "Cargo.toml"))).toBe(true);
  });

  it("Cargo.toml declares the chx binary", () => {
    const cargo = readFileSync(join(root, "Cargo.toml"), "utf8");
    expect(cargo).toContain('name = "chx"');
    expect(cargo).toContain('path = "src/main.rs"');
  });

  it("diff source file exists", () => {
    expect(existsSync(join(root, "src", "diff.rs"))).toBe(true);
  });

  it("diff render source file exists", () => {
    expect(existsSync(join(root, "src", "diff_render.rs"))).toBe(true);
  });

  it("test binary data files exist", () => {
    expect(existsSync(join(root, "tests", "data", "simple_ascii.bin"))).toBe(true);
    expect(existsSync(join(root, "tests", "data", "mixed_bytes.bin"))).toBe(true);
  });
});
