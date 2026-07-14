---
change: CHG-0005-preserve-progressive-provenance-while-making-specsync-warnings-blocking-inside-t
artifact: design
---

# Design

Use one immutable Trust action. Keep `profile = \"standard\"` and `provenance.mode = \"soft\"`; place `specsync check --strict --force --require-coverage 100` first in `fledge.toml`'s verify lane.
