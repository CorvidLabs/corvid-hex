//! Integration tests for memory-mapped large file support.
//!
//! These tests create sparse files at or above the 100MB mmap threshold
//! to verify that the mmap code path works correctly.

use std::io::Write;

/// Helper: create a temp file of a given size by writing a header and seeking.
/// Returns the path as a String (the file is kept alive via the NamedTempFile).
fn create_large_file(size: u64) -> tempfile::NamedTempFile {
    let mut tmp = tempfile::NamedTempFile::new().unwrap();

    // Write a recognizable header
    let header = b"CHX_TEST_HEADER_";
    tmp.write_all(header).unwrap();

    // Seek to near the end and write a footer to make the file the desired size
    let footer = b"_END";
    let padding_len = size as usize - header.len() - footer.len();

    // Write zeros in chunks to avoid huge allocations
    let chunk = vec![0u8; 64 * 1024];
    let mut remaining = padding_len;
    while remaining > 0 {
        let n = remaining.min(chunk.len());
        tmp.write_all(&chunk[..n]).unwrap();
        remaining -= n;
    }
    tmp.write_all(footer).unwrap();
    tmp.flush().unwrap();
    tmp
}

#[test]
fn mmap_open_and_read() {
    // Create a file just above the 100MB threshold
    let size = 100 * 1024 * 1024 + 1024; // 100MB + 1KB
    let tmp = create_large_file(size);
    let path = tmp.path().to_str().unwrap();

    // Open it — should use mmap
    // We can't call is_mapped() from outside the crate, but we can verify
    // it opened without error and reads correctly.
    let buf = chx::Buffer::open(path).unwrap();

    assert_eq!(buf.len(), size as usize);
    assert!(!buf.is_empty());

    // Read the header
    assert_eq!(buf.get(0), Some(b'C'));
    assert_eq!(buf.get(1), Some(b'H'));
    assert_eq!(buf.get(2), Some(b'X'));

    // Read zeros in the middle
    assert_eq!(buf.get(1024 * 1024), Some(0));

    // Read the footer
    let end = size as usize;
    assert_eq!(buf.get(end - 4), Some(b'_'));
    assert_eq!(buf.get(end - 3), Some(b'E'));
    assert_eq!(buf.get(end - 2), Some(b'N'));
    assert_eq!(buf.get(end - 1), Some(b'D'));

    // Out of bounds
    assert_eq!(buf.get(end), None);
}

#[test]
fn mmap_edit_and_undo() {
    let size = 100 * 1024 * 1024 + 512;
    let tmp = create_large_file(size);
    let path = tmp.path().to_str().unwrap();

    let mut buf = chx::Buffer::open(path).unwrap();

    // Edit near the beginning
    buf.set(0, 0xFF);
    assert_eq!(buf.get(0), Some(0xFF));
    assert!(buf.is_dirty());

    // Edit near the end
    let near_end = size as usize - 10;
    buf.set(near_end, 0xAB);
    assert_eq!(buf.get(near_end), Some(0xAB));

    // Undo both
    buf.undo();
    assert_eq!(buf.get(near_end), Some(0));
    buf.undo();
    assert_eq!(buf.get(0), Some(b'C')); // original header byte
    assert!(!buf.is_dirty());

    // Redo
    buf.redo();
    assert_eq!(buf.get(0), Some(0xFF));
}

#[test]
fn mmap_save_persists() {
    let size = 100 * 1024 * 1024 + 256;
    let tmp = create_large_file(size);
    let path = tmp.path().to_str().unwrap().to_string();

    {
        let mut buf = chx::Buffer::open(&path).unwrap();
        buf.set(0, b'Z');
        buf.set(100, 0xBE);
        buf.save().unwrap();
        assert!(!buf.is_dirty());
    }

    // Re-open and verify
    let buf = chx::Buffer::open(&path).unwrap();
    assert_eq!(buf.get(0), Some(b'Z'));
    assert_eq!(buf.get(100), Some(0xBE));
    assert_eq!(buf.len(), size as usize);
}

#[test]
fn mmap_find_works() {
    let size = 100 * 1024 * 1024 + 1024;
    let tmp = create_large_file(size);
    let path = tmp.path().to_str().unwrap();

    let buf = chx::Buffer::open(path).unwrap();

    // Find the header
    assert_eq!(buf.find(b"CHX_TEST", 0), Some(0));

    // Find the footer
    let end = size as usize;
    assert_eq!(buf.find(b"_END", end - 10), Some(end - 4));

    // Pattern not found
    assert_eq!(buf.find(b"NOTHERE", 0), None);
}
