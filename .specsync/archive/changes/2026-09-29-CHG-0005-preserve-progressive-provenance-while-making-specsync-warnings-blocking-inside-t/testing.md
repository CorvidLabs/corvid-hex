---
change: CHG-0005-preserve-progressive-provenance-while-making-specsync-warnings-blocking-inside-t
artifact: testing
---

# Testing

Run released SpecSync 5.0.1 in normal and forced strict modes at 100 percent, all four agent status, `fledge trust doctor`, and `fledge trust verify`. The Trust lane must execute strict SpecSync before Clippy, all 273 Rust unit tests, 4 mmap integration tests, and 40 genuine Bun tests.
