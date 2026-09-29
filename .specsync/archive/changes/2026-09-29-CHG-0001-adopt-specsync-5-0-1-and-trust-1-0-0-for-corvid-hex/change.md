---
id: CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-for-corvid-hex
state: archived
type: migration
base_commit: 06679cad2d50877fe7c455b9e4d9a7ac53478f9e
---

# Adopt SpecSync 5.0.1 and Trust 1.0.0 for Corvid Hex

## Intent

Adopt SpecSync 5.0.1 and Trust 1.0.0 for Corvid Hex

## Affected Canonical Specs

- None

## Acceptance Criteria

- SpecSync remains at 100 percent; Rust clippy, Rust tests, and Bun tests pass; agents and Trust doctor are healthy; release, docs, Atlas, CLI, and library boundaries remain unchanged.

## No-spec Rationale

Governance and CI orchestration plus non-semantic Rust match-guard cleanup required by the current Clippy toolchain; Rust and Bun behavior, public interfaces, release artifacts, documentation, and Atlas publication are unchanged.

## Migration Note

Migrated by hand to SpecSync 6 per Leif's decision (2026-09-28); the 6.0.0 tool refused to archive this legacy record (`` legacy accepted change `CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-for-corvid-hex` requires exactly one distinct valid historical reconstruction, found 0; first reconstruction failure: legacy accepted change `CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-for-corvid-hex` cannot reproduce its signed raw-content aggregate ``).

- Workflow v1 (SpecSync 5) record, accepted on 2026-07-14 by the closing approval already stored in `approvals.json`. SpecSync 6.0.0 reports its accepted evidence as stale, for the reason quoted above.
- Moved by hand from `.specsync/changes/CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-for-corvid-hex/` into the layout `specsync change archive` writes: `accepted-state.json` is the unchanged accepted `state.json`, `state.json` is marked `archived`, and this file's front matter says `archived`.
- `approvals.json`, `verification.json`, `verification-attempts.json` and every other artifact are the original SpecSync 5 evidence, unchanged. `verification.json` verifies commit `308cecbd90eb484cf364546679ad8d02e6623cda`, not the tree this record was archived from.
- This migration added no verification evidence, test result, attempt history, or approval. It is a manual migration, not a fresh re-verification.
- Closing it through the tool takes `specsync change reopen`, `specsync change verify`, then `specsync change accept`, which writes a new closing approval. Per Leif's decision it was archived by hand instead, so no reopen or new approval is recorded.
