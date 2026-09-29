---
id: CHG-0004-preserve-progressive-provenance-while-making-specsync-warnings-blocking-inside-t
state: archived
type: feature
base_commit: 2a9437007a7311568365522d7d80ffa7034bcea7
---

# Preserve progressive provenance while making SpecSync warnings blocking inside the unified Trust lifecycle lane

## Intent

Preserve progressive provenance while making SpecSync warnings blocking inside the unified Trust lifecycle lane

## Affected Canonical Specs

- `chx`

## Acceptance Criteria

- Trust uses the standard profile with soft provenance
- and its Fledge lifecycle runs forced strict SpecSync at 100 percent before native checks so warnings block the unified Trust gate while local and hosted Trust can pass.

## No-spec Rationale

Not applicable

## Migration Note

Migrated by hand to SpecSync 6 per Leif's decision (2026-09-28); the 6.0.0 tool refused to archive this legacy record (`` legacy accepted change `CHG-0004-preserve-progressive-provenance-while-making-specsync-warnings-blocking-inside-t` requires exactly one distinct valid historical reconstruction, found 0; first reconstruction failure: legacy accepted change `CHG-0004-preserve-progressive-provenance-while-making-specsync-warnings-blocking-inside-t` cannot reproduce its signed raw-content aggregate ``).

- Workflow v1 (SpecSync 5) record, accepted on 2026-07-14 by the closing approval already stored in `approvals.json`. SpecSync 6.0.0 reports its accepted evidence as stale, for the reason quoted above.
- Moved by hand from `.specsync/changes/CHG-0004-preserve-progressive-provenance-while-making-specsync-warnings-blocking-inside-t/` into the layout `specsync change archive` writes: `accepted-state.json` is the unchanged accepted `state.json`, `state.json` is marked `archived`, and this file's front matter says `archived`.
- `approvals.json`, `verification.json`, `verification-attempts.json` and every other artifact are the original SpecSync 5 evidence, unchanged. `verification.json` verifies commit `308cecbd90eb484cf364546679ad8d02e6623cda`, not the tree this record was archived from.
- This migration added no verification evidence, test result, attempt history, or approval. It is a manual migration, not a fresh re-verification.
- Closing it through the tool takes `specsync change reopen`, `specsync change verify`, then `specsync change accept`, which writes a new closing approval. Per Leif's decision it was archived by hand instead, so no reopen or new approval is recorded.
