---
id: CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-for-corvid-hex
state: draft
type: migration
base_commit: 06679cad2d50877fe7c455b9e4d9a7ac53478f9e
---

# Adopt SpecSync 5.0.1 and Trust 1.0.0 for Corvid Hex

## Intent

Adopt SpecSync 5.0.1 and Trust 1.0.0 for Corvid Hex

## Affected Canonical Specs

- None

## Acceptance Criteria

- SpecSync remains at 100 percent; Rust clippy
- Rust tests
- and Bun tests pass; agents and Trust doctor are healthy; release
- docs
- Atlas
- CLI
- and library boundaries remain unchanged.

## No-spec Rationale

Governance and CI orchestration only; Rust and Bun behavior, public interfaces, release artifacts, documentation, and Atlas publication are unchanged.
