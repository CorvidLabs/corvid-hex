---
module: entropy
version: 1
status: draft
files:
  - src/entropy.rs
db_tables: []
depends_on:
  - specs/buffer/buffer.spec.md
---

## Purpose

Provides Shannon entropy calculation and visualization helpers for the hex editor. Computes per-window entropy values across the file buffer, maps entropy to a color gradient for heatmap display, and averages entropy over arbitrary byte ranges.

## Public API

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `entropy_from_counts` | `pub fn entropy_from_counts(counts: &[u32; 256], total: usize) -> f64` | Calculates Shannon entropy from a pre-computed byte frequency table. Returns a value in [0.0, 8.0] bits per byte. Returns 0.0 if `total` is 0. |
| `calculate_entropy` | `pub fn calculate_entropy(data: &[u8]) -> f64` | Calculates Shannon entropy for a byte slice. Returns a value in [0.0, 8.0] bits per byte. Delegates to `entropy_from_counts`. |
| `calculate_window_entropies` | `pub fn calculate_window_entropies(buffer: &Buffer, window_size: usize) -> Vec<f64>` | Splits the buffer into fixed-size windows and returns the entropy of each. Uses `Buffer::count_bytes_in_range` to respect the edit overlay. Returns an empty vec if the buffer is empty or window size is 0. |
| `entropy_color` | `pub fn entropy_color(entropy: f64) -> Color` | Maps an entropy value [0.0, 8.0] to a gradient color. Blue = low entropy (structured), Red = high entropy (random/encrypted). |
| `average_entropy_for_range` | `pub fn average_entropy_for_range(windows: &[f64], window_size: usize, start: usize, end: usize) -> f64` | Averages pre-computed window entropy values for the windows overlapping the byte range [start, end). Returns 0.0 for empty/invalid inputs. |

## Invariants

1. `calculate_entropy` returns exactly 0.0 for empty input and for uniform single-value input.
2. `calculate_entropy` returns exactly 8.0 (within floating-point precision) for input containing all 256 distinct byte values in equal proportion.
3. `entropy_color` clamps the input to [0.0, 8.0] before mapping.
4. `calculate_window_entropies` produces exactly `ceil(buffer.len() / window_size)` entries.
5. `average_entropy_for_range` never panics for out-of-bounds ranges; it clamps and returns 0.0 for fully out-of-bounds inputs.

## Behavioral Examples

**Uniform data has zero entropy**
- Given: 256 bytes all set to `0xAA`
- When: `calculate_entropy` is called
- Then: returns 0.0

**Maximum entropy for all byte values**
- Given: exactly one of each byte value 0x00–0xFF (256 bytes)
- When: `calculate_entropy` is called
- Then: returns ~8.0

**Color gradient endpoints**
- Given: entropy = 0.0
- When: `entropy_color` is called
- Then: returns a blue-ish `Color::Rgb` with red = 0

- Given: entropy = 8.0
- When: `entropy_color` is called
- Then: returns a red-ish `Color::Rgb` with red > 100 and green < 50

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Empty buffer passed to `calculate_window_entropies` | Returns empty `Vec` |
| `window_size` of 0 passed to `calculate_window_entropies` | Returns empty `Vec` |
| Empty windows slice passed to `average_entropy_for_range` | Returns 0.0 |
| `start >= end` passed to `average_entropy_for_range` | Returns 0.0 |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `crate::buffer::Buffer` | Provides `count_bytes_in_range` for edit-aware byte frequency counting |
| `ratatui::style::Color` | Return type for `entropy_color` |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-30 | Initial spec |
