use anyhow::{Context, Result};
use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

/// Size threshold above which we use memory-mapped I/O instead of reading
/// the entire file into memory. 100 MB.
const MMAP_THRESHOLD: u64 = 100 * 1024 * 1024;

/// Backing store for file data — either fully in-memory or memory-mapped.
enum Backing {
    /// Small files: entire contents loaded into a Vec.
    InMemory(Vec<u8>),
    /// Large files: memory-mapped for on-demand paging by the OS.
    Mapped(Mmap),
}

impl Backing {
    fn len(&self) -> usize {
        match self {
            Backing::InMemory(v) => v.len(),
            Backing::Mapped(m) => m.len(),
        }
    }

    fn get(&self, offset: usize) -> Option<u8> {
        match self {
            Backing::InMemory(v) => v.get(offset).copied(),
            Backing::Mapped(m) => {
                if offset < m.len() {
                    Some(m[offset])
                } else {
                    None
                }
            }
        }
    }

    fn is_mapped(&self) -> bool {
        matches!(self, Backing::Mapped(_))
    }
}

struct UndoEntry {
    offset: usize,
    old_value: Option<u8>,
}

/// Copy-on-open buffer with edit overlay.
/// Original data is either read into memory (small files) or memory-mapped
/// (large files). Edits are stored in a HashMap overlay in both cases.
pub struct Buffer {
    pub path: PathBuf,
    backing: Backing,
    edits: HashMap<usize, u8>,
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
}

impl Buffer {
    pub fn open(path: &str) -> Result<Self> {
        let path = Path::new(path);
        let backing = if path.exists() {
            let metadata = fs::metadata(path)
                .with_context(|| format!("Failed to stat {}", path.display()))?;
            let file_size = metadata.len();

            if file_size >= MMAP_THRESHOLD {
                let file = File::open(path)
                    .with_context(|| format!("Failed to open {}", path.display()))?;
                // SAFETY: The file is opened read-only. We handle edits via the
                // overlay HashMap, so the mmap is never written to directly.
                // The file must not be truncated by another process while mapped.
                let mmap = unsafe { Mmap::map(&file) }
                    .with_context(|| format!("Failed to mmap {}", path.display()))?;
                Backing::Mapped(mmap)
            } else {
                let data = fs::read(path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;
                Backing::InMemory(data)
            }
        } else {
            Backing::InMemory(Vec::new())
        };

        Ok(Self {
            path: path.to_path_buf(),
            backing,
            edits: HashMap::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.backing.len()
    }

    pub fn is_empty(&self) -> bool {
        self.backing.len() == 0
    }

    /// Returns true if the buffer is backed by a memory-mapped file.
    #[cfg(test)]
    pub fn is_mapped(&self) -> bool {
        self.backing.is_mapped()
    }

    /// Get byte at offset, returning edited value if present.
    pub fn get(&self, offset: usize) -> Option<u8> {
        if offset >= self.backing.len() {
            return None;
        }
        Some(
            self.edits
                .get(&offset)
                .copied()
                .unwrap_or_else(|| self.backing.get(offset).unwrap()),
        )
    }

    /// Get the original (unedited) byte at offset.
    #[cfg(test)]
    fn original_byte(&self, offset: usize) -> Option<u8> {
        self.backing.get(offset)
    }

    /// Set a byte at the given offset (overwrite mode only).
    pub fn set(&mut self, offset: usize, value: u8) {
        if offset < self.backing.len() {
            let old_edit = self.edits.get(&offset).copied();
            let original = self.backing.get(offset).unwrap();
            let old_value = old_edit.unwrap_or(original);
            if value == old_value {
                return;
            }
            self.undo_stack.push(UndoEntry {
                offset,
                old_value: old_edit,
            });
            self.redo_stack.clear();
            if value == original {
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
            Some(v) => {
                self.edits.insert(entry.offset, v);
            }
            None => {
                self.edits.remove(&entry.offset);
            }
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
            Some(v) => {
                self.edits.insert(entry.offset, v);
            }
            None => {
                self.edits.remove(&entry.offset);
            }
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
        if self.backing.is_mapped() {
            // For mmap-backed files, we apply edits by reading the mmap,
            // patching in edits, and writing to a temporary file, then
            // renaming over the original. We stream in chunks to avoid
            // loading the entire file into a Vec.
            self.save_mapped()?;
        } else {
            self.save_in_memory()?;
        }
        Ok(())
    }

    fn save_in_memory(&mut self) -> Result<()> {
        let original = match &self.backing {
            Backing::InMemory(v) => v,
            Backing::Mapped(_) => unreachable!(),
        };
        let mut data = original.clone();
        for (&offset, &value) in &self.edits {
            if offset < data.len() {
                data[offset] = value;
            }
        }
        fs::write(&self.path, &data)
            .with_context(|| format!("Failed to write {}", self.path.display()))?;
        self.backing = Backing::InMemory(data);
        self.edits.clear();
        Ok(())
    }

    fn save_mapped(&mut self) -> Result<()> {
        use std::io::{BufWriter, Write};

        let dir = self.path.parent().unwrap_or(Path::new("."));
        let tmp_path = dir.join(format!(
            ".chx-tmp-{}",
            std::process::id()
        ));

        let len = self.backing.len();
        const CHUNK_SIZE: usize = 64 * 1024; // 64 KB chunks

        {
            let file = File::create(&tmp_path)
                .with_context(|| format!("Failed to create temp file {}", tmp_path.display()))?;
            let mut writer = BufWriter::new(file);

            let mut pos = 0;
            while pos < len {
                let end = (pos + CHUNK_SIZE).min(len);
                // Build chunk, applying edits
                let mut chunk: Vec<u8> = Vec::with_capacity(end - pos);
                for offset in pos..end {
                    let byte = self
                        .edits
                        .get(&offset)
                        .copied()
                        .unwrap_or_else(|| self.backing.get(offset).unwrap());
                    chunk.push(byte);
                }
                writer.write_all(&chunk)
                    .with_context(|| format!("Failed to write to {}", tmp_path.display()))?;
                pos = end;
            }
            writer.flush()?;
        }

        // Drop the mmap before renaming so the file handle is released
        // (important on some platforms).
        self.backing = Backing::InMemory(Vec::new());

        fs::rename(&tmp_path, &self.path)
            .with_context(|| format!("Failed to rename temp file to {}", self.path.display()))?;

        // Re-open as mmap if still large, otherwise in-memory
        let metadata = fs::metadata(&self.path)?;
        if metadata.len() >= MMAP_THRESHOLD {
            let file = File::open(&self.path)?;
            let mmap = unsafe { Mmap::map(&file) }?;
            self.backing = Backing::Mapped(mmap);
        } else {
            let data = fs::read(&self.path)?;
            self.backing = Backing::InMemory(data);
        }

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

    #[test]
    fn small_file_uses_in_memory() {
        let buf = make_buffer(b"small file data");
        assert!(!buf.is_mapped());
    }

    #[test]
    fn mmap_threshold_constant() {
        // Verify the threshold is 100MB
        assert_eq!(MMAP_THRESHOLD, 100 * 1024 * 1024);
    }

    #[test]
    fn original_byte_returns_unedited() {
        let mut buf = make_buffer(b"ABCD");
        buf.set(0, 0xFF);
        // get() returns the edit
        assert_eq!(buf.get(0), Some(0xFF));
        // original_byte() returns the original
        assert_eq!(buf.original_byte(0), Some(b'A'));
    }

    #[test]
    fn edits_work_identically_on_in_memory() {
        let mut buf = make_buffer(b"ABCDEFGH");
        assert!(!buf.is_mapped());

        // Edit, undo, redo cycle
        buf.set(0, 0x01);
        buf.set(4, 0x02);
        assert_eq!(buf.get(0), Some(0x01));
        assert_eq!(buf.get(4), Some(0x02));
        assert!(buf.is_dirty());

        buf.undo();
        assert_eq!(buf.get(4), Some(b'E'));
        buf.undo();
        assert_eq!(buf.get(0), Some(b'A'));
        assert!(!buf.is_dirty());

        buf.redo();
        assert_eq!(buf.get(0), Some(0x01));
        buf.redo();
        assert_eq!(buf.get(4), Some(0x02));
    }

    #[test]
    fn save_in_memory_then_reopen() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"ORIGINAL").unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        let mut buf = Buffer::open(&path).unwrap();
        assert!(!buf.is_mapped());
        buf.set(0, b'M');
        buf.set(1, b'O');
        buf.set(2, b'D');
        buf.save().unwrap();
        assert!(!buf.is_dirty());

        // Verify on disk
        let buf2 = Buffer::open(&path).unwrap();
        assert_eq!(buf2.get(0), Some(b'M'));
        assert_eq!(buf2.get(1), Some(b'O'));
        assert_eq!(buf2.get(2), Some(b'D'));
        assert_eq!(buf2.get(3), Some(b'G'));
    }
}
