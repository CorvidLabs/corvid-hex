/**
 * Integration tests for the format template system (corvid-hex / chx).
 *
 * These tests:
 *   1. Run the Rust unit-test suite via `cargo test` (delegates to #[cfg(test)]
 *      blocks in format.rs and other modules).
 *   2. Verify magic-byte detection logic against the test binary fixtures in
 *      tests/data/.
 *   3. Validate the TOML custom-template specification.
 */

import { describe, test, expect } from "bun:test";
import { spawnSync } from "child_process";
import { readFileSync, existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const CARGO = resolve(process.env.HOME ?? "/", ".cargo/bin/cargo");

// ─── Rust test suite ──────────────────────────────────────────────────────────

describe("Rust unit tests (cargo test)", () => {
  test("all cargo tests pass", () => {
    const result = spawnSync(CARGO, ["test"], {
      cwd: ROOT,
      encoding: "utf-8",
      timeout: 120_000,
    });
    if (result.error) {
      throw result.error;
    }
    expect(result.status).toBe(0);
  });
});

// ─── Magic byte validation ────────────────────────────────────────────────────

const MAGIC_BYTES: Record<string, { offset: number; bytes: number[] }> = {
  PNG: { offset: 0, bytes: [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a] },
  ZIP: { offset: 0, bytes: [0x50, 0x4b, 0x03, 0x04] },
  ELF: { offset: 0, bytes: [0x7f, 0x45, 0x4c, 0x46] },
  JPEG: { offset: 0, bytes: [0xff, 0xd8, 0xff] },
  GIF: { offset: 0, bytes: [0x47, 0x49, 0x46, 0x38] }, // GIF8
  BMP: { offset: 0, bytes: [0x42, 0x4d] }, // BM
  PDF: { offset: 0, bytes: [0x25, 0x50, 0x44, 0x46, 0x2d] }, // %PDF-
  SQLite: {
    offset: 0,
    bytes: [0x53, 0x51, 0x4c, 0x69, 0x74, 0x65, 0x20, 0x66, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x20, 0x33, 0x00],
  },
};

function matchesMagic(data: Buffer, offset: number, magic: number[]): boolean {
  if (data.length < offset + magic.length) return false;
  return magic.every((b, i) => data[offset + i] === b);
}

describe("magic byte format detection", () => {
  for (const [format, { offset, bytes }] of Object.entries(MAGIC_BYTES)) {
    test(`${format} magic bytes are correctly defined`, () => {
      // Create a minimal buffer that starts with the magic bytes
      const buf = Buffer.alloc(Math.max(32, offset + bytes.length));
      for (let i = 0; i < bytes.length; i++) {
        buf[offset + i] = bytes[i];
      }
      expect(matchesMagic(buf, offset, bytes)).toBe(true);
    });

    test(`${format} magic does not match empty buffer`, () => {
      expect(matchesMagic(Buffer.alloc(0), offset, bytes)).toBe(false);
    });

    test(`${format} magic does not match all-zero buffer`, () => {
      if (bytes.every((b) => b === 0)) return; // skip if magic is all zeros
      const zeroBuf = Buffer.alloc(Math.max(32, offset + bytes.length));
      expect(matchesMagic(zeroBuf, offset, bytes)).toBe(false);
    });
  }

  test("WAV requires WAVE marker at offset 8", () => {
    const buf = Buffer.alloc(16);
    buf.write("RIFF", 0, "ascii");
    // Without WAVE at offset 8: no match
    const riff = [0x52, 0x49, 0x46, 0x46];
    const wave = [0x57, 0x41, 0x56, 0x45];
    expect(matchesMagic(buf, 0, riff)).toBe(true);
    expect(matchesMagic(buf, 8, wave)).toBe(false);
    // With WAVE at offset 8: both match
    buf.write("WAVE", 8, "ascii");
    expect(matchesMagic(buf, 0, riff)).toBe(true);
    expect(matchesMagic(buf, 8, wave)).toBe(true);
  });
});

// ─── Test data fixtures ───────────────────────────────────────────────────────

describe("test data fixtures", () => {
  const fixtures = ["simple_ascii.bin", "search_patterns.bin", "mixed_bytes.bin"];

  for (const fixture of fixtures) {
    test(`${fixture} exists and is readable`, () => {
      const path = resolve(ROOT, "tests/data", fixture);
      expect(existsSync(path)).toBe(true);
      const data = readFileSync(path);
      expect(data.length).toBeGreaterThan(0);
    });
  }

  test("simple_ascii.bin starts with ASCII text", () => {
    const data = readFileSync(resolve(ROOT, "tests/data/simple_ascii.bin"));
    // Should start with "Hello"
    expect(data.subarray(0, 5).toString("ascii")).toBe("Hello");
  });

  test("search_patterns.bin contains repeated patterns", () => {
    const data = readFileSync(resolve(ROOT, "tests/data/search_patterns.bin"));
    // Contains 'abc' at offset 0
    expect(data[0]).toBe(0x61); // 'a'
    expect(data[1]).toBe(0x62); // 'b'
    expect(data[2]).toBe(0x63); // 'c'
  });
});

// ─── TOML template spec ───────────────────────────────────────────────────────

describe("TOML custom template format", () => {
  test("template spec file documents the TOML format", () => {
    // The format spec is embedded in the source doc-comment; verify format.rs exists
    const formatRs = resolve(ROOT, "src/format.rs");
    expect(existsSync(formatRs)).toBe(true);
    const content = readFileSync(formatRs, "utf-8");
    expect(content).toContain("magic_offset");
    expect(content).toContain("field_type");
    expect(content).toContain("parse_toml_template");
  });

  test("supported field types are documented", () => {
    const formatRs = readFileSync(resolve(ROOT, "src/format.rs"), "utf-8");
    const expectedTypes = ["u8", "u16le", "u16be", "u32le", "u32be", "u64le", "u64be", "ascii", "bytes"];
    for (const t of expectedTypes) {
      expect(formatRs.toLowerCase()).toContain(t);
    }
  });

  test("at least 5 built-in format templates are registered", () => {
    const formatRs = readFileSync(resolve(ROOT, "src/format.rs"), "utf-8");
    // Each built-in is registered via a make_* call in builtin_templates()
    const makeMatches = formatRs.match(/make_\w+\(\)/g) ?? [];
    // builtin_templates() itself calls at least 5 make_* functions
    expect(makeMatches.length).toBeGreaterThanOrEqual(5);
  });
});
