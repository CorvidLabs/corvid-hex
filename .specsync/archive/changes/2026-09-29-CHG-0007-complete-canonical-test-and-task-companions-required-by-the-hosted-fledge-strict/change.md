---
id: CHG-0007-complete-canonical-test-and-task-companions-required-by-the-hosted-fledge-strict
state: archived
type: documentation
base_commit: 308cecbd90eb484cf364546679ad8d02e6623cda
---

# Complete canonical test and task companions required by the hosted Fledge strict structural gate

## Intent

Complete canonical test and task companions required by the hosted Fledge strict structural gate

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

- Every canonical module has complete truthful task and testing companions
- Fledge strict structural validation reports zero warnings
- and released SpecSync remains at 100 percent coverage without changing product behavior.

## No-spec Rationale

Not applicable

## Migration Note

Migrated by hand to SpecSync 6 per Leif's decision (2026-09-28); the 6.0.0 tool refused to archive this legacy record (`` delivery input `specs/buffer/buffer.spec.md` (owner `buffer`) changed after acceptance and no accepted or archived successor change covers it; run `specsync change reopen CHG-0007-complete-canonical-test-and-task-companions-required-by-the-hosted-fledge-strict` to re-verify the accepted change ``).

- Workflow v1 (SpecSync 5) record, accepted on 2026-07-14 by the closing approval already stored in `approvals.json`. SpecSync 6.0.0 reports its accepted evidence as stale, for the reason quoted above.
- Moved by hand from `.specsync/changes/CHG-0007-complete-canonical-test-and-task-companions-required-by-the-hosted-fledge-strict/` into the layout `specsync change archive` writes: `accepted-state.json` is the unchanged accepted `state.json`, `state.json` is marked `archived`, and this file's front matter says `archived`.
- `approvals.json`, `verification.json`, `verification-attempts.json` and every other artifact are the original SpecSync 5 evidence, unchanged. `verification.json` verifies commit `308cecbd90eb484cf364546679ad8d02e6623cda`, not the tree this record was archived from.
- This migration added no verification evidence, test result, attempt history, or approval. It is a manual migration, not a fresh re-verification.
- Closing it through the tool takes `specsync change reopen`, `specsync change verify`, then `specsync change accept`, which writes a new closing approval. Per Leif's decision it was archived by hand instead, so no reopen or new approval is recorded.
