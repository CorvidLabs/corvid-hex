---
id: CHG-0003-complete-the-specsync-5-and-trust-1-migration-with-strict-trust-enforcement-com
state: archived
type: feature
base_commit: 2a9437007a7311568365522d7d80ffa7034bcea7
---

# Complete the SpecSync 5 and Trust 1 migration with strict Trust enforcement, complete governance-path coverage, and stable IDs for every existing requirement companion

## Intent

Complete the SpecSync 5 and Trust 1 migration with strict Trust enforcement, complete governance-path coverage, and stable IDs for every existing requirement companion

## Affected Canonical Specs

- `buffer`
- `chx`
- `diff`
- `diff_render`
- `entropy`
- `format`
- `input`
- `inspector`
- `render`
- `search`

## Acceptance Criteria

- Trust uses the strict profile; every authoritative Trust and lifecycle configuration path is meaningful; every existing acceptance criterion has a deterministic stable requirement ID and unchanged semantics; released SpecSync 5.0.1 passes normal and forced strict at 100 percent; all four agent integrations
- Trust doctor
- local Trust
- Clippy
- Rust tests
- Bun tests
- and exact-head hosted checks pass.

## No-spec Rationale

Not applicable

## Migration Note

Migrated by hand to SpecSync 6 per Leif's decision (2026-09-28); the 6.0.0 tool refused to archive this legacy record (`` legacy accepted change `CHG-0003-complete-the-specsync-5-and-trust-1-migration-with-strict-trust-enforcement-com` requires exactly one distinct valid historical reconstruction, found 0; first reconstruction failure: legacy accepted change `CHG-0003-complete-the-specsync-5-and-trust-1-migration-with-strict-trust-enforcement-com` cannot reproduce its signed raw-content aggregate ``).

- Workflow v1 (SpecSync 5) record, accepted on 2026-07-14 by the closing approval already stored in `approvals.json`. SpecSync 6.0.0 reports its accepted evidence as stale, for the reason quoted above.
- Moved by hand from `.specsync/changes/CHG-0003-complete-the-specsync-5-and-trust-1-migration-with-strict-trust-enforcement-com/` into the layout `specsync change archive` writes: `accepted-state.json` is the unchanged accepted `state.json`, `state.json` is marked `archived`, and this file's front matter says `archived`.
- `approvals.json`, `verification.json`, `verification-attempts.json` and every other artifact are the original SpecSync 5 evidence, unchanged. `verification.json` verifies commit `308cecbd90eb484cf364546679ad8d02e6623cda`, not the tree this record was archived from.
- This migration added no verification evidence, test result, attempt history, or approval. It is a manual migration, not a fresh re-verification.
- Closing it through the tool takes `specsync change reopen`, `specsync change verify`, then `specsync change accept`, which writes a new closing approval. Per Leif's decision it was archived by hand instead, so no reopen or new approval is recorded.
