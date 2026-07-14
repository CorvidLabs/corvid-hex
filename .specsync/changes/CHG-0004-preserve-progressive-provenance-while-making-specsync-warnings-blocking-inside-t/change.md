---
id: CHG-0004-preserve-progressive-provenance-while-making-specsync-warnings-blocking-inside-t
state: accepted
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
