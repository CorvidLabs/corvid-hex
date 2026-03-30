use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Copy-on-open buffer with edit overlay.
/// Original data is read into memory; edits are stored in a HashMap overlay.
pub struct Buffer {
    pub path: PathBuf,
    original: Vec<u8>,
    edits: HashMap<usize, u8>,
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
            if value == self.original[offset] {
                self.edits.remove(&offset);
            } else {
                self.edits.insert(offset, value);
            }
        }
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
