---
id: CHG-0005-preserve-progressive-provenance-while-making-specsync-warnings-blocking-inside-t
state: accepted
type: migration
base_commit: 2a9437007a7311568365522d7d80ffa7034bcea7
---

# Preserve progressive provenance while making SpecSync warnings blocking inside the unified Trust lifecycle lane

## Intent

Preserve progressive provenance while making SpecSync warnings blocking inside the unified Trust lifecycle lane

## Affected Canonical Specs

- None

## Acceptance Criteria

- Trust uses the standard profile with soft provenance
- and its Fledge lifecycle runs forced strict SpecSync at 100 percent before native checks so warnings block the unified Trust gate while local and hosted Trust can pass.

## No-spec Rationale

This corrects Trust and Fledge governance only; canonical Corvid Hex behavior, requirements, source, tests, dependencies, and public interfaces remain unchanged.
