use crate::buffer::Buffer;
use ratatui::style::Color;

/// Calculate Shannon entropy from a frequency table and total byte count.
/// Returns a value in [0.0, 8.0] bits per byte.
pub fn entropy_from_counts(counts: &[u32; 256], total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let len = total as f64;
    let mut entropy = 0.0f64;
    for &count in counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Calculate Shannon entropy for a byte slice.
/// Returns a value in [0.0, 8.0] bits per byte.
#[allow(dead_code)]
pub fn calculate_entropy(data: &[u8]) -> f64 {
    let mut counts = [0u32; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    entropy_from_counts(&counts, data.len())
}

/// Calculate entropy for each window across the buffer.
/// Window size is in bytes. Returns one f64 per window, in order.
pub fn calculate_window_entropies(buffer: &Buffer, window_size: usize) -> Vec<f64> {
    let len = buffer.len();
    if len == 0 || window_size == 0 {
        return Vec::new();
    }
    let num_windows = len.div_ceil(window_size);
    let mut result = Vec::with_capacity(num_windows);

    for i in 0..num_windows {
        let start = i * window_size;
        let end = (start + window_size).min(len);
        let counts = buffer.count_bytes_in_range(start, end);
        let window_len = end - start;
        result.push(entropy_from_counts(&counts, window_len));
    }

    result
}

/// Map entropy value [0.0, 8.0] to a display color.
/// Blue = low entropy (structured/repetitive), Red = high entropy (random/encrypted).
pub fn entropy_color(entropy: f64) -> Color {
    let t = (entropy / 8.0).clamp(0.0, 1.0);
    if t < 0.25 {
        // Low entropy: dark blue to blue
        let s = t / 0.25;
        Color::Rgb(0, 0, (80.0 + s * 175.0) as u8)
    } else if t < 0.5 {
        // Medium-low: blue to cyan
        let s = (t - 0.25) / 0.25;
        Color::Rgb(0, (s * 200.0) as u8, (255.0 - s * 55.0) as u8)
    } else if t < 0.75 {
        // Medium-high: cyan to yellow
        let s = (t - 0.5) / 0.25;
        Color::Rgb((s * 220.0) as u8, 200, (200.0 * (1.0 - s)) as u8)
    } else {
        // High entropy: yellow to red
        let s = (t - 0.75) / 0.25;
        Color::Rgb(220, (200.0 * (1.0 - s)) as u8, 0)
    }
}

/// Average the entropy values for windows that overlap the byte range [start, end).
pub fn average_entropy_for_range(
    windows: &[f64],
    window_size: usize,
    start: usize,
    end: usize,
) -> f64 {
    if windows.is_empty() || window_size == 0 || start >= end {
        return 0.0;
    }
    let first = start / window_size;
    let last = (end.saturating_sub(1)) / window_size;
    let last = last.min(windows.len().saturating_sub(1));
    if first >= windows.len() {
        return 0.0;
    }
    let slice = &windows[first..=last];
    slice.iter().sum::<f64>() / slice.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_bytes_max_entropy() {
        // All 256 distinct byte values — maximum possible entropy.
        let data: Vec<u8> = (0..=255u8).collect();
        let e = calculate_entropy(&data);
        assert!((e - 8.0).abs() < 0.001, "Expected ~8.0, got {e}");
    }

    #[test]
    fn single_byte_value_zero_entropy() {
        let data = vec![0xAA; 256];
        let e = calculate_entropy(&data);
        assert_eq!(e, 0.0, "All same bytes should have 0 entropy");
    }

    #[test]
    fn empty_slice_zero_entropy() {
        assert_eq!(calculate_entropy(&[]), 0.0);
    }

    #[test]
    fn two_equal_values_one_bit() {
        // Equal mix of two values = 1 bit entropy.
        let mut data = vec![0x00u8; 128];
        data.extend(vec![0xFFu8; 128]);
        let e = calculate_entropy(&data);
        assert!((e - 1.0).abs() < 0.001, "Expected ~1.0, got {e}");
    }

    #[test]
    fn four_equal_values_two_bits() {
        // Equal mix of four values = 2 bits entropy.
        let mut data = Vec::new();
        for b in [0x00u8, 0x11, 0x22, 0x33] {
            data.extend(vec![b; 64]);
        }
        let e = calculate_entropy(&data);
        assert!((e - 2.0).abs() < 0.001, "Expected ~2.0, got {e}");
    }

    #[test]
    fn entropy_color_low_is_blue() {
        match entropy_color(0.0) {
            Color::Rgb(r, _, b) => {
                assert_eq!(r, 0, "Low entropy should have r=0");
                assert!(b > 50, "Low entropy should have high blue");
            }
            c => panic!("Expected Rgb color, got {c:?}"),
        }
    }

    #[test]
    fn entropy_color_high_is_reddish() {
        match entropy_color(8.0) {
            Color::Rgb(r, g, _) => {
                assert!(r > 100, "High entropy should have high red");
                assert!(g < 50, "High entropy should have low green");
            }
            c => panic!("Expected Rgb color, got {c:?}"),
        }
    }

    #[test]
    fn entropy_color_mid_is_warm() {
        // Middle entropy (4.0 out of 8.0) should be somewhere in the cyan-yellow range.
        match entropy_color(4.0) {
            Color::Rgb(_, g, _) => {
                assert!(g > 100, "Mid entropy should have noticeable green component");
            }
            c => panic!("Expected Rgb color, got {c:?}"),
        }
    }

    #[test]
    fn average_entropy_single_window() {
        let windows = vec![4.0f64, 6.0, 2.0];
        let avg = average_entropy_for_range(&windows, 256, 0, 256);
        assert!((avg - 4.0).abs() < 0.001, "Expected 4.0, got {avg}");
    }

    #[test]
    fn average_entropy_all_windows() {
        let windows = vec![2.0f64, 6.0, 4.0];
        let avg = average_entropy_for_range(&windows, 256, 0, 768);
        assert!((avg - 4.0).abs() < 0.001, "Expected 4.0, got {avg}");
    }

    #[test]
    fn average_entropy_empty_windows() {
        let avg = average_entropy_for_range(&[], 256, 0, 256);
        assert_eq!(avg, 0.0);
    }
}
