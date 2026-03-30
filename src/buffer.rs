use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

struct UndoEntry {
    offset: usize,
    old_value: Option<u8>,
}

/// Copy-on-open buffer with edit overlay.
/// Original data is read into memory; edits are stored in a HashMap overlay.
pub struct Buffer {
    pub path: PathBuf,
    original: Vec<u8>,
    edits: HashMap<usize, u8>,
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
}

impl Buffer {
    pub fn open(path: &str) -> Result<Self> {
        let path = Path::new(path);
        let original = if path.exists() {
            fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?
        } else {
            Vec::new()
        };
        Ok(Self {
            path: path.to_path_buf(),
            original,
            edits: HashMap::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.original.len()
    }

    pub fn is_empty(&self) -> bool {
        self.original.is_empty()
    }

    /// Get byte at offset, returning edited value if present.
    pub fn get(&self, offset: usize) -> Option<u8> {
        if offset >= self.original.len() {
            return None;
        }
        Some(self.edits.get(&offset).copied().unwrap_or(self.original[offset]))
    }

    /// Set a byte at the given offset (overwrite mode only).
    pub fn set(&mut self, offset: usize, value: u8) {
        if offset < self.original.len() {
            let old_edit = self.edits.get(&offset).copied();
            let old_value = old_edit.unwrap_or(self.original[offset]);
            if value == old_value {
                return;
            }
            self.undo_stack.push(UndoEntry {
                offset,
                old_value: old_edit,
            });
            self.redo_stack.clear();
            if value == self.original[offset] {
                self.edits.remove(&offset);
            } else {
                self.edits.insert(offset, value);
            }
        }
    }

    /// Undo the last edit. Returns the offset that was changed, if any.
    pub fn undo(&mut self) -> Option<usize> {
        let entry = self.undo_stack.pop()?;
        self.redo_stack.push(UndoEntry {
            offset: entry.offset,
            old_value: self.edits.get(&entry.offset).copied(),
        });
        match entry.old_value {
            Some(v) => { self.edits.insert(entry.offset, v); }
            None => { self.edits.remove(&entry.offset); }
        }
        Some(entry.offset)
    }

    /// Redo the last undone edit. Returns the offset that was changed, if any.
    pub fn redo(&mut self) -> Option<usize> {
        let entry = self.redo_stack.pop()?;
        self.undo_stack.push(UndoEntry {
            offset: entry.offset,
            old_value: self.edits.get(&entry.offset).copied(),
        });
        match entry.old_value {
            Some(v) => { self.edits.insert(entry.offset, v); }
            None => { self.edits.remove(&entry.offset); }
        }
        Some(entry.offset)
    }

    pub fn is_modified(&self, offset: usize) -> bool {
        self.edits.contains_key(&offset)
    }

    pub fn is_dirty(&self) -> bool {
        !self.edits.is_empty()
    }

    /// Save the buffer to disk.
    pub fn save(&mut self) -> Result<()> {
        let mut data = self.original.clone();
        for (&offset, &value) in &self.edits {
            if offset < data.len() {
                data[offset] = value;
            }
        }
        fs::write(&self.path, &data)
            .with_context(|| format!("Failed to write {}", self.path.display()))?;
        self.original = data;
        self.edits.clear();
        Ok(())
    }

    /// Search for a byte pattern starting from an offset.
    pub fn find(&self, pattern: &[u8], start: usize) -> Option<usize> {
        if pattern.is_empty() || self.is_empty() {
            return None;
        }
        let len = self.len();
        for i in start..len {
            if i + pattern.len() > len {
                break;
            }
            let mut matched = true;
            for (j, &p) in pattern.iter().enumerate() {
                if self.get(i + j) != Some(p) {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Some(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_buffer(data: &[u8]) -> Buffer {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(data).unwrap();
        Buffer::open(tmp.path().to_str().unwrap()).unwrap()
    }

    #[test]
    fn open_reads_file() {
        let buf = make_buffer(b"Hello");
        assert_eq!(buf.len(), 5);
        assert!(!buf.is_empty());
        assert_eq!(buf.get(0), Some(b'H'));
        assert_eq!(buf.get(4), Some(b'o'));
        assert_eq!(buf.get(5), None);
    }

    #[test]
    fn open_nonexistent_creates_empty() {
        let buf = Buffer::open("/tmp/chx_test_nonexistent_file_xyz").unwrap();
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert_eq!(buf.get(0), None);
    }

    #[test]
    fn set_and_get_edit() {
        let mut buf = make_buffer(b"ABCD");
        assert!(!buf.is_dirty());

        buf.set(1, 0xFF);
        assert_eq!(buf.get(1), Some(0xFF));
        assert!(buf.is_modified(1));
        assert!(buf.is_dirty());
        assert!(!buf.is_modified(0));
    }

    #[test]
    fn set_same_value_removes_edit() {
        let mut buf = make_buffer(b"ABCD");
        buf.set(0, 0xFF);
        assert!(buf.is_modified(0));

        // Set back to original value
        buf.set(0, b'A');
        assert!(!buf.is_modified(0));
        assert!(!buf.is_dirty());
    }

    #[test]
    fn set_out_of_bounds_ignored() {
        let mut buf = make_buffer(b"AB");
        buf.set(5, 0xFF);
        assert!(!buf.is_dirty());
    }

    #[test]
    fn save_persists_edits() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"Hello").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let mut buf = Buffer::open(&path).unwrap();
        buf.set(0, b'J');
        buf.save().unwrap();

        assert!(!buf.is_dirty());
        assert_eq!(buf.get(0), Some(b'J'));

        // Re-read from disk
        let buf2 = Buffer::open(&path).unwrap();
        assert_eq!(buf2.get(0), Some(b'J'));
    }

    #[test]
    fn find_ascii() {
        let buf = make_buffer(b"Hello World");
        assert_eq!(buf.find(b"World", 0), Some(6));
        assert_eq!(buf.find(b"Hello", 0), Some(0));
        assert_eq!(buf.find(b"xyz", 0), None);
    }

    #[test]
    fn find_respects_start_offset() {
        let buf = make_buffer(b"abcabc");
        assert_eq!(buf.find(b"abc", 0), Some(0));
        assert_eq!(buf.find(b"abc", 1), Some(3));
        assert_eq!(buf.find(b"abc", 4), None);
    }

    #[test]
    fn find_empty_pattern_returns_none() {
        let buf = make_buffer(b"data");
        assert_eq!(buf.find(b"", 0), None);
    }

    #[test]
    fn find_on_empty_buffer() {
        let buf = Buffer::open("/tmp/chx_test_nonexistent_xyz").unwrap();
        assert_eq!(buf.find(b"x", 0), None);
    }

    #[test]
    fn find_sees_edits() {
        let mut buf = make_buffer(b"AAAA");
        buf.set(2, b'B');
        buf.set(3, b'B');
        assert_eq!(buf.find(b"BB", 0), Some(2));
    }

    #[test]
    fn undo_reverts_edit() {
        let mut buf = make_buffer(b"ABCD");
        buf.set(0, 0xFF);
        assert_eq!(buf.get(0), Some(0xFF));
        assert!(buf.is_dirty());

        let offset = buf.undo();
        assert_eq!(offset, Some(0));
        assert_eq!(buf.get(0), Some(b'A'));
        assert!(!buf.is_dirty());
    }

    #[test]
    fn redo_restores_edit() {
        let mut buf = make_buffer(b"ABCD");
        buf.set(0, 0xFF);
        buf.undo();
        assert_eq!(buf.get(0), Some(b'A'));

        let offset = buf.redo();
        assert_eq!(offset, Some(0));
        assert_eq!(buf.get(0), Some(0xFF));
        assert!(buf.is_dirty());
    }

    #[test]
    fn undo_empty_returns_none() {
        let mut buf = make_buffer(b"AB");
        assert_eq!(buf.undo(), None);
    }

    #[test]
    fn redo_empty_returns_none() {
        let mut buf = make_buffer(b"AB");
        assert_eq!(buf.redo(), None);
    }

    #[test]
    fn new_edit_clears_redo_stack() {
        let mut buf = make_buffer(b"ABCD");
        buf.set(0, 0xFF);
        buf.undo();
        // New edit should clear redo
        buf.set(1, 0xEE);
        assert_eq!(buf.redo(), None);
    }

    #[test]
    fn multiple_undo_redo() {
        let mut buf = make_buffer(b"ABCD");
        buf.set(0, 0x01);
        buf.set(1, 0x02);
        buf.set(2, 0x03);

        buf.undo(); // revert offset 2
        assert_eq!(buf.get(2), Some(b'C'));
        buf.undo(); // revert offset 1
        assert_eq!(buf.get(1), Some(b'B'));

        buf.redo(); // restore offset 1
        assert_eq!(buf.get(1), Some(0x02));
        buf.redo(); // restore offset 2
        assert_eq!(buf.get(2), Some(0x03));
    }

    #[test]
    fn set_same_value_no_undo_entry() {
        let mut buf = make_buffer(b"AB");
        buf.set(0, b'A'); // same as original — no-op
        assert_eq!(buf.undo(), None);
    }
}
