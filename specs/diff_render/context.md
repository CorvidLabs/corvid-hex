---
spec: diff_render.spec.md
---

## Key Decisions

- Layout-driven: Ratatui constraints split screen into header, stats bar, dual-panel split view, and status bar.
- Dynamic bytes-per-row: auto-fitted via `(half_width - 10) / 3`.
- Hierarchical styling: cursor (inverted white) > diff highlight (red) > additions (green bold) > semantic byte color (null/printable/control).
- XOR mode renders XOR of left/right bytes on-the-fly in the right panel.
- Stateless `draw_diff()` function — no cached render output, full redraw per frame.

## Files to Read First

- `src/diff_render.rs` — all diff view rendering, color logic, layout

## Current Status

Complete. Split-pane rendering, XOR view, dynamic layout, and color-coded byte categories all working. Launched in v0.2.0.

## Notes

- `byte_color()` classifies bytes: null (dark gray), printable ASCII (cyan), control (yellow).
- Status bar uses three-part layout: mode label, centered message, right-aligned hex address.
