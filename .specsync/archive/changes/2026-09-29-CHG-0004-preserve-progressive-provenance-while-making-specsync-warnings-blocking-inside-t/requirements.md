---
change: CHG-0004-preserve-progressive-provenance-while-making-specsync-warnings-blocking-inside-t
artifact: requirements
---

# Requirements

1. The unified Trust gate SHALL keep provenance in soft mode until a signed remote ledger exists.\n2. The Trust-managed lifecycle SHALL run released SpecSync 5.0.1 with `--strict --force --require-coverage 100`.\n3. Strict SpecSync SHALL run before Clippy, Rust tests, and genuine Bun tests.\n4. No product, public API, package, dependency, release, documentation, or Atlas behavior SHALL change.
