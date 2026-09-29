---
id: CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act
state: archived
type: migration
base_commit: 3c9ef4eb1c03c1bb643206fb9509d0d0c17a03a3
---

# Use Fledge-native strict spec validation in hosted Trust before the SpecSync action installs its binary

## Intent

Use Fledge-native strict spec validation in hosted Trust before the SpecSync action installs its binary

## Affected Canonical Specs

- None

## Acceptance Criteria

- The Trust-managed Fledge lifecycle runs `fledge spec check --strict` before native checks on clean hosted runners
- the later pinned SpecSync 5.0.1 contract enforces 100 percent coverage
- and exact-head Trust passes with progressive provenance.

## No-spec Rationale

This is a hosted-runner portability correction to Fledge lifecycle orchestration; product behavior, canonical requirements, source, tests, packages, dependencies, and public interfaces remain unchanged.

## Migration Note

Migrated by hand to SpecSync 6 per Leif's decision (2026-09-28); the 6.0.0 tool refused to archive this legacy record (`` legacy accepted change `CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act` requires exactly one distinct valid historical reconstruction, found 0; first reconstruction failure: legacy accepted change `CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act` cannot reproduce its signed raw-content aggregate ``).

- Workflow v1 (SpecSync 5) record, accepted on 2026-07-14 by the closing approval already stored in `approvals.json`. SpecSync 6.0.0 reports its accepted evidence as stale, for the reason quoted above.
- Moved by hand from `.specsync/changes/CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act/` into the layout `specsync change archive` writes: `accepted-state.json` is the unchanged accepted `state.json`, `state.json` is marked `archived`, and this file's front matter says `archived`.
- `approvals.json`, `verification.json`, `verification-attempts.json` and every other artifact are the original SpecSync 5 evidence, unchanged. `verification.json` verifies commit `308cecbd90eb484cf364546679ad8d02e6623cda`, not the tree this record was archived from.
- This migration added no verification evidence, test result, attempt history, or approval. It is a manual migration, not a fresh re-verification.
- Closing it through the tool takes `specsync change reopen`, `specsync change verify`, then `specsync change accept`, which writes a new closing approval. Per Leif's decision it was archived by hand instead, so no reopen or new approval is recorded.
