---
id: CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act
state: accepted
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
