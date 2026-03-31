use std::fs;
use std::path::Path;

/// Represents the result of comparing two files byte-by-byte.
pub struct DiffState {
    pub left_data: Vec<u8>,
    pub right_data: Vec<u8>,
    pub left_name: String,
    pub right_name: String,
    /// Sorted list of offsets where the files differ (including extra bytes).
    pub diff_offsets: Vec<usize>,
    /// Current index into diff_offsets for navigation.
    pub diff_index: usize,
    /// Cursor position (byte offset).
    pub cursor: usize,
    pub scroll_offset: usize,
    pub bytes_per_row: usize,
    pub visible_rows: usize,
    pub status_message: Option<String>,
    /// Whether to show XOR view instead of raw bytes on the right.
    pub xor_view: bool,
}

/// Summary statistics for a diff.
#[allow(dead_code)]
pub struct DiffStats {
    pub total_bytes: usize,
    pub diff_count: usize,
    pub match_percentage: f64,
    pub first_diff: Option<usize>,
    pub left_size: usize,
    pub right_size: usize,
}

impl DiffState {
    pub fn open(left_path: &str, right_path: &str) -> anyhow::Result<Self> {
        let left_data = fs::read(Path::new(left_path))
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", left_path, e))?;
        let right_data = fs::read(Path::new(right_path))
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", right_path, e))?;

        let left_name = Path::new(left_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| left_path.to_string());
        let right_name = Path::new(right_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| right_path.to_string());

        let diff_offsets = compute_diff_offsets(&left_data, &right_data);

        Ok(Self {
            left_data,
            right_data,
            left_name,
            right_name,
            diff_offsets,
            diff_index: 0,
            cursor: 0,
            scroll_offset: 0,
            bytes_per_row: 16,
            visible_rows: 24,
            status_message: None,
            xor_view: false,
        })
    }

    /// The maximum length across both files.
    pub fn max_len(&self) -> usize {
        self.left_data.len().max(self.right_data.len())
    }

    pub fn stats(&self) -> DiffStats {
        let max_len = self.max_len();
        let diff_count = self.diff_offsets.len();
        let match_percentage = if max_len == 0 {
            100.0
        } else {
            ((max_len - diff_count) as f64 / max_len as f64) * 100.0
        };
        DiffStats {
            total_bytes: max_len,
            diff_count,
            match_percentage,
            first_diff: self.diff_offsets.first().copied(),
            left_size: self.left_data.len(),
            right_size: self.right_data.len(),
        }
    }

    pub fn cursor_row(&self) -> usize {
        self.cursor / self.bytes_per_row
    }

    pub fn ensure_cursor_visible(&mut self) {
        let row = self.cursor_row();
        if row < self.scroll_offset {
            self.scroll_offset = row;
        } else if row >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = row - self.visible_rows + 1;
        }
    }

    pub fn move_cursor(&mut self, offset: isize) {
        let max = if self.max_len() == 0 {
            0
        } else {
            self.max_len() - 1
        };
        let new_pos = self.cursor as isize + offset;
        self.cursor = new_pos.clamp(0, max as isize) as usize;
        self.ensure_cursor_visible();
    }

    pub fn move_cursor_to(&mut self, pos: usize) {
        let max = if self.max_len() == 0 {
            0
        } else {
            self.max_len() - 1
        };
        self.cursor = pos.min(max);
        self.ensure_cursor_visible();
    }

    pub fn page_down(&mut self) {
        let jump = self.visible_rows * self.bytes_per_row;
        self.move_cursor(jump as isize);
    }

    pub fn page_up(&mut self) {
        let jump = self.visible_rows * self.bytes_per_row;
        self.move_cursor(-(jump as isize));
    }

    /// Jump to next difference (`]c`).
    pub fn next_diff(&mut self) {
        if self.diff_offsets.is_empty() {
            self.status_message = Some("No differences".to_string());
            return;
        }
        // Find first diff offset after cursor
        match self.diff_offsets.binary_search(&(self.cursor + 1)) {
            Ok(idx) => {
                self.diff_index = idx;
                self.move_cursor_to(self.diff_offsets[idx]);
            }
            Err(idx) => {
                if idx < self.diff_offsets.len() {
                    self.diff_index = idx;
                    self.move_cursor_to(self.diff_offsets[idx]);
                } else {
                    // Wrap around
                    self.diff_index = 0;
                    self.move_cursor_to(self.diff_offsets[0]);
                    self.status_message = Some("Wrapped to first difference".to_string());
                }
            }
        }
        let stats = self.stats();
        if self.status_message.is_none() {
            self.status_message = Some(format!(
                "Diff {}/{} at 0x{:08X}",
                self.diff_index + 1,
                stats.diff_count,
                self.cursor,
            ));
        }
    }

    /// Jump to previous difference (`[c`).
    pub fn prev_diff(&mut self) {
        if self.diff_offsets.is_empty() {
            self.status_message = Some("No differences".to_string());
            return;
        }
        // Find last diff offset before cursor
        match self.diff_offsets.binary_search(&self.cursor) {
            Ok(idx) => {
                if idx > 0 {
                    self.diff_index = idx - 1;
                    self.move_cursor_to(self.diff_offsets[self.diff_index]);
                } else {
                    // Wrap around
                    self.diff_index = self.diff_offsets.len() - 1;
                    self.move_cursor_to(self.diff_offsets[self.diff_index]);
                    self.status_message = Some("Wrapped to last difference".to_string());
                }
            }
            Err(idx) => {
                if idx > 0 {
                    self.diff_index = idx - 1;
                    self.move_cursor_to(self.diff_offsets[self.diff_index]);
                } else {
                    // Wrap around
                    self.diff_index = self.diff_offsets.len() - 1;
                    self.move_cursor_to(self.diff_offsets[self.diff_index]);
                    self.status_message = Some("Wrapped to last difference".to_string());
                }
            }
        }
        let stats = self.stats();
        if self.status_message.is_none() {
            self.status_message = Some(format!(
                "Diff {}/{} at 0x{:08X}",
                self.diff_index + 1,
                stats.diff_count,
                self.cursor,
            ));
        }
    }

    pub fn toggle_xor_view(&mut self) {
        self.xor_view = !self.xor_view;
        self.status_message = Some(if self.xor_view {
            "XOR view enabled".to_string()
        } else {
            "XOR view disabled".to_string()
        });
    }

    /// Get the left byte at an offset, or None if beyond the file.
    pub fn left_byte(&self, offset: usize) -> Option<u8> {
        self.left_data.get(offset).copied()
    }

    /// Get the right byte at an offset, or None if beyond the file.
    pub fn right_byte(&self, offset: usize) -> Option<u8> {
        self.right_data.get(offset).copied()
    }

    /// Check if the given offset is a diff.
    pub fn is_diff(&self, offset: usize) -> bool {
        self.diff_offsets.binary_search(&offset).is_ok()
    }
}

/// Compute sorted list of offsets where the two byte slices differ.
fn compute_diff_offsets(left: &[u8], right: &[u8]) -> Vec<usize> {
    let max_len = left.len().max(right.len());
    let mut offsets = Vec::new();
    for i in 0..max_len {
        let l = left.get(i);
        let r = right.get(i);
        if l != r {
            offsets.push(i);
        }
    }
    offsets
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp(data: &[u8]) -> NamedTempFile {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(data).unwrap();
        tmp
    }

    #[test]
    fn identical_files_no_diffs() {
        let left = write_temp(b"ABCDEF");
        let right = write_temp(b"ABCDEF");
        let state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert!(state.diff_offsets.is_empty());
        let stats = state.stats();
        assert_eq!(stats.diff_count, 0);
        assert!((stats.match_percentage - 100.0).abs() < f64::EPSILON);
        assert_eq!(stats.first_diff, None);
    }

    #[test]
    fn single_byte_difference() {
        let left = write_temp(b"ABCDEF");
        let right = write_temp(b"ABXDEF");
        let state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(state.diff_offsets, vec![2]);
        let stats = state.stats();
        assert_eq!(stats.diff_count, 1);
        assert_eq!(stats.first_diff, Some(2));
    }

    #[test]
    fn different_sizes_extra_bytes_are_diffs() {
        let left = write_temp(b"AB");
        let right = write_temp(b"ABCD");
        let state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(state.diff_offsets, vec![2, 3]);
        assert_eq!(state.max_len(), 4);
        let stats = state.stats();
        assert_eq!(stats.left_size, 2);
        assert_eq!(stats.right_size, 4);
    }

    #[test]
    fn completely_different() {
        let left = write_temp(b"\x00\x00\x00");
        let right = write_temp(b"\xFF\xFF\xFF");
        let state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(state.diff_offsets, vec![0, 1, 2]);
        let stats = state.stats();
        assert!((stats.match_percentage - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn navigate_next_diff() {
        let left = write_temp(b"AAXAAXAA");
        let right = write_temp(b"AAYAAYAA");
        let mut state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(state.diff_offsets, vec![2, 5]);

        state.next_diff();
        assert_eq!(state.cursor, 2);

        state.next_diff();
        assert_eq!(state.cursor, 5);

        // Wrap around
        state.next_diff();
        assert_eq!(state.cursor, 2);
    }

    #[test]
    fn navigate_prev_diff() {
        let left = write_temp(b"AAXAAXAA");
        let right = write_temp(b"AAYAAYAA");
        let mut state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();

        // Start at beginning, prev should wrap to last
        state.prev_diff();
        assert_eq!(state.cursor, 5);

        state.prev_diff();
        assert_eq!(state.cursor, 2);

        // Wrap again
        state.prev_diff();
        assert_eq!(state.cursor, 5);
    }

    #[test]
    fn navigate_no_diffs() {
        let left = write_temp(b"ABC");
        let right = write_temp(b"ABC");
        let mut state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();

        state.next_diff();
        assert!(state.status_message.as_ref().unwrap().contains("No differences"));
        state.prev_diff();
        assert!(state.status_message.as_ref().unwrap().contains("No differences"));
    }

    #[test]
    fn is_diff_check() {
        let left = write_temp(b"ABCD");
        let right = write_temp(b"AXCX");
        let state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert!(!state.is_diff(0));
        assert!(state.is_diff(1));
        assert!(!state.is_diff(2));
        assert!(state.is_diff(3));
    }

    #[test]
    fn xor_view_toggle() {
        let left = write_temp(b"AB");
        let right = write_temp(b"AB");
        let mut state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert!(!state.xor_view);
        state.toggle_xor_view();
        assert!(state.xor_view);
        state.toggle_xor_view();
        assert!(!state.xor_view);
    }

    #[test]
    fn cursor_movement() {
        let left = write_temp(&vec![0u8; 256]);
        let right = write_temp(&vec![0u8; 256]);
        let mut state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();

        state.move_cursor(5);
        assert_eq!(state.cursor, 5);
        state.move_cursor(-3);
        assert_eq!(state.cursor, 2);
        state.move_cursor(-100);
        assert_eq!(state.cursor, 0);
        state.move_cursor(1000);
        assert_eq!(state.cursor, 255);
    }

    #[test]
    fn page_navigation() {
        let left = write_temp(&vec![0u8; 4096]);
        let right = write_temp(&vec![0u8; 4096]);
        let mut state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        state.visible_rows = 4;

        state.page_down();
        assert_eq!(state.cursor, 4 * 16);
        state.page_up();
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn empty_files() {
        let left = write_temp(b"");
        let right = write_temp(b"");
        let state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(state.max_len(), 0);
        assert!(state.diff_offsets.is_empty());
    }

    #[test]
    fn one_empty_one_not() {
        let left = write_temp(b"");
        let right = write_temp(b"ABC");
        let state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(state.diff_offsets, vec![0, 1, 2]);
        assert_eq!(state.max_len(), 3);
    }

    #[test]
    fn left_byte_right_byte() {
        let left = write_temp(b"AB");
        let right = write_temp(b"XY");
        let state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(state.left_byte(0), Some(b'A'));
        assert_eq!(state.right_byte(0), Some(b'X'));
        assert_eq!(state.left_byte(2), None);
    }

    #[test]
    fn stats_match_percentage() {
        // 4 bytes, 1 diff → 75% match
        let left = write_temp(b"ABCD");
        let right = write_temp(b"ABXD");
        let state = DiffState::open(
            left.path().to_str().unwrap(),
            right.path().to_str().unwrap(),
        )
        .unwrap();
        let stats = state.stats();
        assert!((stats.match_percentage - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_diff_offsets_direct() {
        let offsets = compute_diff_offsets(b"ABCD", b"AXCX");
        assert_eq!(offsets, vec![1, 3]);
    }

    #[test]
    fn compute_diff_offsets_different_lengths() {
        let offsets = compute_diff_offsets(b"AB", b"ABCD");
        assert_eq!(offsets, vec![2, 3]);
    }
}
