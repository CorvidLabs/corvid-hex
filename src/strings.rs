use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringKind {
    Ascii,
    Utf8,
    Utf16Le,
    Utf16Be,
}

impl StringKind {
    pub fn label(&self) -> &'static str {
        match self {
            StringKind::Ascii => "ASCII",
            StringKind::Utf8 => "UTF-8",
            StringKind::Utf16Le => "UTF-16LE",
            StringKind::Utf16Be => "UTF-16BE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StringEntry {
    pub offset: usize,
    pub length: usize,
    pub kind: StringKind,
    pub text: String,
}

/// Extract strings from binary data.
///
/// Scans for:
/// - ASCII/UTF-8: contiguous runs of printable characters (bytes 0x20–0x7E, tab, newline,
///   carriage return, and valid UTF-8 multibyte sequences for non-ASCII printable chars)
/// - UTF-16 LE: two-byte little-endian sequences of printable BMP characters
/// - UTF-16 BE: two-byte big-endian sequences of printable BMP characters
///
/// Only strings with at least `min_length` characters are included.
/// Results are sorted by offset.
pub fn extract_strings(data: &[u8], min_length: usize) -> Vec<StringEntry> {
    let mut results = Vec::new();
    extract_utf8_strings(data, min_length, &mut results);
    extract_utf16_strings(data, min_length, false, &mut results);
    extract_utf16_strings(data, min_length, true, &mut results);
    results.sort_by_key(|e| e.offset);
    results
}

fn extract_utf8_strings(data: &[u8], min_length: usize, results: &mut Vec<StringEntry>) {
    let mut i = 0;
    while i < data.len() {
        let start = i;
        let mut text = String::new();
        let mut has_multibyte = false;

        loop {
            if i >= data.len() {
                break;
            }
            let b = data[i];

            if b < 0x80 {
                // ASCII byte
                if (b >= 0x20 && b <= 0x7E) || b == b'\t' || b == b'\n' || b == b'\r' {
                    text.push(b as char);
                    i += 1;
                } else {
                    break;
                }
            } else {
                // Multibyte UTF-8 sequence
                let seq_len = if b & 0xE0 == 0xC0 {
                    2
                } else if b & 0xF0 == 0xE0 {
                    3
                } else if b & 0xF8 == 0xF0 {
                    4
                } else {
                    // Invalid lead byte
                    break;
                };

                if i + seq_len > data.len() {
                    break;
                }

                // Validate continuation bytes
                let valid_cont = (1..seq_len).all(|k| data[i + k] & 0xC0 == 0x80);
                if !valid_cont {
                    break;
                }

                if let Ok(s) = std::str::from_utf8(&data[i..i + seq_len]) {
                    if let Some(c) = s.chars().next() {
                        if !c.is_control() {
                            text.push(c);
                            has_multibyte = true;
                            i += seq_len;
                            continue;
                        }
                    }
                }
                break;
            }
        }

        if text.chars().count() >= min_length {
            results.push(StringEntry {
                offset: start,
                length: i - start,
                kind: if has_multibyte {
                    StringKind::Utf8
                } else {
                    StringKind::Ascii
                },
                text,
            });
        }

        // Ensure we always make forward progress
        if i == start {
            i += 1;
        }
    }
}

fn extract_utf16_strings(
    data: &[u8],
    min_length: usize,
    big_endian: bool,
    results: &mut Vec<StringEntry>,
) {
    let mut i = 0;
    while i + 1 < data.len() {
        let start = i;
        let mut text = String::new();

        loop {
            if i + 1 >= data.len() {
                break;
            }

            let code_unit = if big_endian {
                u16::from_be_bytes([data[i], data[i + 1]])
            } else {
                u16::from_le_bytes([data[i], data[i + 1]])
            };

            // Accept printable characters: basic ASCII range and most BMP characters
            // (excluding control chars, surrogates 0xD800–0xDFFF, and noncharacters)
            let printable = (code_unit >= 0x0020 && code_unit <= 0x007E)
                || (code_unit >= 0x00A0 && code_unit <= 0xD7FF)
                || (code_unit >= 0xE000 && code_unit <= 0xFFFD);

            if printable {
                if let Some(c) = char::from_u32(code_unit as u32) {
                    if !c.is_control() {
                        text.push(c);
                        i += 2;
                        continue;
                    }
                }
            }
            break;
        }

        if text.chars().count() >= min_length {
            results.push(StringEntry {
                offset: start,
                length: i - start,
                kind: if big_endian {
                    StringKind::Utf16Be
                } else {
                    StringKind::Utf16Le
                },
                text,
            });
        }

        // Ensure forward progress
        if i == start {
            i += 1;
        }
    }
}

/// Export strings list to a text file.
/// Each line: `0xOFFSET\tKIND\tTEXT`
pub fn export_strings(entries: &[StringEntry], path: &Path) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    for entry in entries {
        writeln!(
            file,
            "0x{:08X}\t{}\t{}",
            entry.offset,
            entry.kind.label(),
            entry.text
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple_ascii() {
        let data = b"\x00\x00Hello World\x00\x00";
        let results = extract_strings(data, 4);
        let ascii: Vec<_> = results.iter().filter(|e| e.kind == StringKind::Ascii).collect();
        assert_eq!(ascii.len(), 1);
        assert_eq!(ascii[0].offset, 2);
        assert_eq!(ascii[0].text, "Hello World");
    }

    #[test]
    fn min_length_filter_keeps_long() {
        let data = b"Hi there big string";
        let results = extract_strings(data, 4);
        let ascii: Vec<_> = results.iter().filter(|e| e.kind == StringKind::Ascii).collect();
        assert!(!ascii.is_empty());
        assert_eq!(ascii[0].text, "Hi there big string");
    }

    #[test]
    fn extract_multiple_ascii() {
        let data = b"Hello\x00\x00\x00World";
        let results = extract_strings(data, 4);
        let ascii: Vec<_> = results.iter().filter(|e| e.kind == StringKind::Ascii).collect();
        assert_eq!(ascii.len(), 2);
        assert_eq!(ascii[0].text, "Hello");
        assert_eq!(ascii[1].text, "World");
    }

    #[test]
    fn min_length_excludes_short() {
        // "Hi" is 2 chars (excluded), "Hello" is 5 chars (included)
        let data = b"\x00Hi\x00Hello\x00";
        let results = extract_strings(data, 4);
        let ascii: Vec<_> = results.iter().filter(|e| e.kind == StringKind::Ascii).collect();
        assert_eq!(ascii.len(), 1);
        assert_eq!(ascii[0].text, "Hello");
    }

    #[test]
    fn extract_utf16_le() {
        // "TEST" encoded as UTF-16 LE
        let data: Vec<u8> = "TEST"
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let results = extract_strings(&data, 4);
        let utf16le: Vec<_> = results
            .iter()
            .filter(|e| e.kind == StringKind::Utf16Le)
            .collect();
        assert_eq!(utf16le.len(), 1);
        assert_eq!(utf16le[0].text, "TEST");
    }

    #[test]
    fn extract_utf16_be() {
        // "TEST" encoded as UTF-16 BE
        let data: Vec<u8> = "TEST"
            .encode_utf16()
            .flat_map(|u| u.to_be_bytes())
            .collect();
        let results = extract_strings(&data, 4);
        let utf16be: Vec<_> = results
            .iter()
            .filter(|e| e.kind == StringKind::Utf16Be)
            .collect();
        assert_eq!(utf16be.len(), 1);
        assert_eq!(utf16be[0].text, "TEST");
    }

    #[test]
    fn empty_data() {
        assert!(extract_strings(&[], 4).is_empty());
    }

    #[test]
    fn all_null_bytes() {
        let data = vec![0u8; 100];
        assert!(extract_strings(&data, 4).is_empty());
    }

    #[test]
    fn results_sorted_by_offset() {
        // Two ASCII strings; scan order should produce sorted results
        let data = b"AAAA\x00BBBB";
        let results = extract_strings(data, 4);
        let ascii: Vec<_> = results.iter().filter(|e| e.kind == StringKind::Ascii).collect();
        assert!(ascii.len() >= 2);
        assert!(ascii[0].offset < ascii[1].offset);
    }

    #[test]
    fn utf8_multibyte_string() {
        // "café" contains a non-ASCII UTF-8 character
        let data = "café".as_bytes();
        let results = extract_strings(data, 4);
        let utf8: Vec<_> = results.iter().filter(|e| e.kind == StringKind::Utf8).collect();
        assert_eq!(utf8.len(), 1);
        assert_eq!(utf8[0].text, "café");
    }

    #[test]
    fn export_creates_file() {
        let entries = vec![StringEntry {
            offset: 0x100,
            length: 5,
            kind: StringKind::Ascii,
            text: "Hello".to_string(),
        }];
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_strings(&entries, tmp.path()).unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("0x00000100"));
        assert!(content.contains("Hello"));
        assert!(content.contains("ASCII"));
    }

    #[test]
    fn export_empty_list() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        export_strings(&[], tmp.path()).unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.is_empty());
    }
}
