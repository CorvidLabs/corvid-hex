---
spec: entropy.spec.md
---

## Key Decisions

- Frequency table approach: pre-allocated `[u32; 256]` array rather than HashMap — O(1) lookup, cache-friendly.
- Shannon entropy formula: `-Σ(p_i * log₂(p_i))` yielding [0.0, 8.0] bits per byte.
- Window-based analysis: sliding window computes per-window entropy; range queries average over overlapping windows.
- Color gradient: entropy → smooth RGB via four-segment piecewise interpolation (blue → cyan → yellow → red).
- Accepts generic `Buffer` trait rather than raw slices for reuse across data sources.

## Files to Read First

- `src/entropy.rs` — entropy calculation, window analysis, color mapping

## Current Status

Complete. Global, per-window, and range-averaged entropy calculations all working. Heatmap color gradient rendering integrated. Launched in v0.2.0.

## Notes

- No caching — recalculates on demand (suitable for small/medium files).
- Tests validate mathematical properties: uniform distribution = 8.0 bits, single byte = 0.0, two equal values = 1.0 bit.
