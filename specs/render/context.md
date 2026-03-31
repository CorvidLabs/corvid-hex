---
spec: render.spec.md
---

## Key Decisions

- Strict color priority: cursor > search hit > modification/category color. Enforced identically in hex and ASCII columns.
- Dynamic bytes-per-row: auto-fit to terminal width via `max_bpr = (width - 14) / 4`. User-requested value clamped to available space.
- Visible rows recalculated per frame: `terminal_height - 3` (header + status + margin), clamped via saturating subtraction.
- Template field overlay uses cyclic palette (8 alternating fg/bg pairs) when `app.show_template_overlay` is true.
- Inspector panel rendered as optional right column showing field interpretations (u8/u16/u32/u64 in LE/BE).

## Files to Read First

- `src/render.rs` — all UI rendering (600+ lines)

## Current Status

Complete. Renders header (with dirty marker), hex/ASCII grid with proper spacing, cursor/search/selection highlighting, status bar, and optional panels (inspector, entropy, strings).

## Notes

- Entropy and strings panels are optional left/bottom overlays toggled by the user.
- The render module is purely presentational — all state lives in `App`.
