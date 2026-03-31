//! Format template system for parsing and labeling known binary file formats.
//!
//! Templates define named fields at specific offsets with types and endianness.
//! Built-in templates cover common formats (PNG, ZIP, ELF, PE, SQLite, JPEG,
//! GIF, BMP, WAV, PDF, Mach-O). Users can add TOML templates in
//! `~/.config/chx/templates/`.
//!
//! # Custom template format (TOML)
//!
//! ```toml
//! name = "My Format"
//! magic = [0xDE, 0xAD, 0xBE, 0xEF]
//! magic_offset = 0
//!
//! [[fields]]
//! name = "Header Magic"
//! offset = 0
//! size = 4
//! field_type = "bytes"
//!
//! [[fields]]
//! name = "Version"
//! offset = 4
//! size = 2
//! field_type = "u16le"
//! ```
//!
//! Supported field types: `u8`, `u16le`, `u16be`, `u32le`, `u32be`,
//! `u64le`, `u64be`, `ascii`, `bytes`.

use serde::Deserialize;
use std::collections::HashMap;

// ─── Field type ──────────────────────────────────────────────────────────────

/// How a template field's bytes are interpreted for display.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    U8,
    U16Le,
    U16Be,
    U32Le,
    U32Be,
    U64Le,
    U64Be,
    AsciiStr,
    Bytes,
}

impl FieldType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "u8" => FieldType::U8,
            "u16le" | "u16" => FieldType::U16Le,
            "u16be" => FieldType::U16Be,
            "u32le" | "u32" => FieldType::U32Le,
            "u32be" => FieldType::U32Be,
            "u64le" | "u64" => FieldType::U64Le,
            "u64be" => FieldType::U64Be,
            "ascii" => FieldType::AsciiStr,
            _ => FieldType::Bytes,
        }
    }
}

// ─── Template field ───────────────────────────────────────────────────────────

/// A single named field within a format template.
#[derive(Debug, Clone)]
pub struct TemplateField {
    pub name: String,
    /// Byte offset from start of file.
    pub offset: usize,
    /// Size in bytes.
    pub size: usize,
    pub field_type: FieldType,
}

impl TemplateField {
    fn new(name: &str, offset: usize, size: usize, field_type: FieldType) -> Self {
        TemplateField {
            name: name.to_string(),
            offset,
            size,
            field_type,
        }
    }
}

/// Parse a field value from a byte slice that starts at the field's offset.
/// `bytes` must be exactly `field.size` bytes (or longer — only the first
/// `field.size` bytes are used).
pub fn parse_field_value(field: &TemplateField, bytes: &[u8]) -> String {
    if bytes.len() < field.size {
        return "(out of range)".to_string();
    }
    let b = &bytes[..field.size];
    match &field.field_type {
        FieldType::U8 => {
            format!("{} (0x{:02X})", b[0], b[0])
        }
        FieldType::U16Le if field.size >= 2 => {
            let v = u16::from_le_bytes([b[0], b[1]]);
            format!("{} (0x{:04X})", v, v)
        }
        FieldType::U16Be if field.size >= 2 => {
            let v = u16::from_be_bytes([b[0], b[1]]);
            format!("{} (0x{:04X})", v, v)
        }
        FieldType::U32Le if field.size >= 4 => {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&b[..4]);
            let v = u32::from_le_bytes(arr);
            format!("{} (0x{:08X})", v, v)
        }
        FieldType::U32Be if field.size >= 4 => {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&b[..4]);
            let v = u32::from_be_bytes(arr);
            format!("{} (0x{:08X})", v, v)
        }
        FieldType::U64Le if field.size >= 8 => {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&b[..8]);
            let v = u64::from_le_bytes(arr);
            format!("{} (0x{:016X})", v, v)
        }
        FieldType::U64Be if field.size >= 8 => {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&b[..8]);
            let v = u64::from_be_bytes(arr);
            format!("{} (0x{:016X})", v, v)
        }
        FieldType::AsciiStr => {
            let s: String = b
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| {
                    if c.is_ascii_graphic() || c == b' ' {
                        c as char
                    } else {
                        '.'
                    }
                })
                .collect();
            format!("\"{}\"", s)
        }
        _ => {
            let hex: Vec<String> = b.iter().map(|x| format!("{:02X}", x)).collect();
            format!("[{}]", hex.join(" "))
        }
    }
}

// ─── Format template ─────────────────────────────────────────────────────────

/// A complete format template describing a binary file format.
#[derive(Debug, Clone)]
pub struct FormatTemplate {
    pub name: String,
    /// Primary magic bytes.
    pub magic: Vec<u8>,
    /// Offset of primary magic bytes.
    pub magic_offset: usize,
    /// Optional secondary magic check (bytes, offset).
    pub second_magic: Option<(Vec<u8>, usize)>,
    /// Static field definitions.
    pub fields: Vec<TemplateField>,
}

impl FormatTemplate {
    /// Returns true if `data` matches this template's magic bytes.
    pub fn matches(&self, data: &[u8]) -> bool {
        if self.magic.is_empty() {
            return false;
        }
        let off = self.magic_offset;
        if data.len() < off + self.magic.len() {
            return false;
        }
        if data[off..off + self.magic.len()] != self.magic[..] {
            return false;
        }
        if let Some((ref m2, off2)) = self.second_magic {
            if data.len() < off2 + m2.len() {
                return false;
            }
            if data[off2..off2 + m2.len()] != m2[..] {
                return false;
            }
        }
        true
    }

    /// Resolve fields, including dynamic chunk fields for PNG and ZIP.
    pub fn resolve_fields(&self, data: &[u8]) -> Vec<TemplateField> {
        let mut fields = self.fields.clone();
        match self.name.as_str() {
            "PNG Image" => fields.extend(resolve_png_chunks(data)),
            "ZIP Archive" => fields.extend(resolve_zip_entries(data)),
            _ => {}
        }
        fields
    }
}

/// Build a byte-offset → (field_name, field_index) lookup map.
/// Each byte within a field's range maps to that field.
pub fn build_field_map(fields: &[TemplateField]) -> HashMap<usize, (String, usize)> {
    let mut map = HashMap::new();
    for (idx, field) in fields.iter().enumerate() {
        for off in field.offset..field.offset + field.size {
            map.entry(off).or_insert_with(|| (field.name.clone(), idx));
        }
    }
    map
}

// ─── Dynamic chunk resolution ─────────────────────────────────────────────────

/// Walk PNG chunks starting after the 8-byte file signature.
/// The static IHDR fields already cover offset 8–0x20, so we skip the first
/// chunk and walk from the next one onward.
fn resolve_png_chunks(data: &[u8]) -> Vec<TemplateField> {
    let mut fields = Vec::new();
    let mut pos = 8usize; // first chunk starts at offset 8 (IHDR)
    let mut n = 0usize;

    while pos + 12 <= data.len() {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(&data[pos..pos + 4]);
        let chunk_data_len = u32::from_be_bytes(arr) as usize;
        let chunk_type = std::str::from_utf8(data.get(pos + 4..pos + 8).unwrap_or(b"????"))
            .unwrap_or("????")
            .to_string();

        // Skip IHDR — already covered by static fields
        if n > 0 {
            let label = format!("PNG {} length", chunk_type);
            fields.push(TemplateField::new(&label, pos, 4, FieldType::U32Be));
            fields.push(TemplateField::new(
                &format!("PNG {} type", chunk_type),
                pos + 4,
                4,
                FieldType::Bytes,
            ));
            let data_end = pos + 8 + chunk_data_len;
            if chunk_data_len > 0 && data_end <= data.len() {
                fields.push(TemplateField::new(
                    &format!("PNG {} data", chunk_type),
                    pos + 8,
                    chunk_data_len,
                    FieldType::Bytes,
                ));
            }
            if data_end + 4 <= data.len() {
                fields.push(TemplateField::new(
                    &format!("PNG {} CRC", chunk_type),
                    data_end,
                    4,
                    FieldType::U32Be,
                ));
            }
        }

        pos += 4 + 4 + chunk_data_len + 4;
        n += 1;
        if n > 64 || chunk_type == "IEND" {
            break;
        }
    }
    fields
}

/// Walk ZIP local file header entries starting at offset 0.
/// Static fields already cover the first entry's fixed header, so we only add
/// dynamic filename fields for entry 0 and full entries for subsequent ones.
fn resolve_zip_entries(data: &[u8]) -> Vec<TemplateField> {
    let mut fields = Vec::new();
    let mut pos = 0usize;
    let mut n = 0usize;

    while pos + 30 <= data.len() {
        if &data[pos..pos + 4] != b"PK\x03\x04" {
            break;
        }
        let fn_len = u16::from_le_bytes([data[pos + 26], data[pos + 27]]) as usize;
        let extra_len = u16::from_le_bytes([data[pos + 28], data[pos + 29]]) as usize;
        let compressed_size =
            u32::from_le_bytes([data[pos + 18], data[pos + 19], data[pos + 20], data[pos + 21]])
                as usize;

        if n > 0 {
            // Already have static fields for entry 0; add full header for the rest
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} Signature", n + 1),
                pos,
                4,
                FieldType::Bytes,
            ));
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} Version Needed", n + 1),
                pos + 4,
                2,
                FieldType::U16Le,
            ));
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} Flags", n + 1),
                pos + 6,
                2,
                FieldType::U16Le,
            ));
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} Compression", n + 1),
                pos + 8,
                2,
                FieldType::U16Le,
            ));
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} CRC-32", n + 1),
                pos + 14,
                4,
                FieldType::U32Le,
            ));
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} Compressed Size", n + 1),
                pos + 18,
                4,
                FieldType::U32Le,
            ));
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} Uncompressed Size", n + 1),
                pos + 22,
                4,
                FieldType::U32Le,
            ));
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} Filename Length", n + 1),
                pos + 26,
                2,
                FieldType::U16Le,
            ));
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} Extra Field Length", n + 1),
                pos + 28,
                2,
                FieldType::U16Le,
            ));
        }
        if fn_len > 0 && pos + 30 + fn_len <= data.len() {
            fields.push(TemplateField::new(
                &format!("ZIP Entry {} Filename", n + 1),
                pos + 30,
                fn_len,
                FieldType::AsciiStr,
            ));
        }

        pos += 30 + fn_len + extra_len + compressed_size;
        n += 1;
        if n > 32 {
            break;
        }
    }
    fields
}

// ─── Built-in templates ───────────────────────────────────────────────────────

fn make_png() -> FormatTemplate {
    FormatTemplate {
        name: "PNG Image".to_string(),
        magic: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("PNG Signature", 0x00, 8, FieldType::Bytes),
            TemplateField::new("IHDR Chunk Length", 0x08, 4, FieldType::U32Be),
            TemplateField::new("IHDR Chunk Type", 0x0C, 4, FieldType::Bytes),
            TemplateField::new("IHDR Width", 0x10, 4, FieldType::U32Be),
            TemplateField::new("IHDR Height", 0x14, 4, FieldType::U32Be),
            TemplateField::new("IHDR Bit Depth", 0x18, 1, FieldType::U8),
            TemplateField::new("IHDR Color Type", 0x19, 1, FieldType::U8),
            TemplateField::new("IHDR Compression", 0x1A, 1, FieldType::U8),
            TemplateField::new("IHDR Filter Method", 0x1B, 1, FieldType::U8),
            TemplateField::new("IHDR Interlace", 0x1C, 1, FieldType::U8),
            TemplateField::new("IHDR CRC", 0x1D, 4, FieldType::U32Be),
        ],
    }
}

fn make_zip() -> FormatTemplate {
    FormatTemplate {
        name: "ZIP Archive".to_string(),
        magic: b"PK\x03\x04".to_vec(),
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("Local File Header Signature", 0x00, 4, FieldType::Bytes),
            TemplateField::new("Version Needed", 0x04, 2, FieldType::U16Le),
            TemplateField::new("General Purpose Bit Flag", 0x06, 2, FieldType::U16Le),
            TemplateField::new("Compression Method", 0x08, 2, FieldType::U16Le),
            TemplateField::new("Last Mod File Time", 0x0A, 2, FieldType::U16Le),
            TemplateField::new("Last Mod File Date", 0x0C, 2, FieldType::U16Le),
            TemplateField::new("CRC-32", 0x0E, 4, FieldType::U32Le),
            TemplateField::new("Compressed Size", 0x12, 4, FieldType::U32Le),
            TemplateField::new("Uncompressed Size", 0x16, 4, FieldType::U32Le),
            TemplateField::new("File Name Length", 0x1A, 2, FieldType::U16Le),
            TemplateField::new("Extra Field Length", 0x1C, 2, FieldType::U16Le),
        ],
    }
}

fn make_elf() -> FormatTemplate {
    FormatTemplate {
        name: "ELF Binary".to_string(),
        magic: vec![0x7F, 0x45, 0x4C, 0x46],
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("ELF Magic", 0x00, 4, FieldType::Bytes),
            TemplateField::new("ELF Class", 0x04, 1, FieldType::U8),
            TemplateField::new("Data Encoding", 0x05, 1, FieldType::U8),
            TemplateField::new("ELF Version", 0x06, 1, FieldType::U8),
            TemplateField::new("OS/ABI", 0x07, 1, FieldType::U8),
            TemplateField::new("ABI Version", 0x08, 1, FieldType::U8),
            TemplateField::new("Padding", 0x09, 7, FieldType::Bytes),
            TemplateField::new("Object Type", 0x10, 2, FieldType::U16Le),
            TemplateField::new("Machine Architecture", 0x12, 2, FieldType::U16Le),
            TemplateField::new("ELF Version (2)", 0x14, 4, FieldType::U32Le),
            TemplateField::new("Entry Point", 0x18, 8, FieldType::U64Le),
            TemplateField::new("Program Header Offset", 0x20, 8, FieldType::U64Le),
            TemplateField::new("Section Header Offset", 0x28, 8, FieldType::U64Le),
            TemplateField::new("Processor Flags", 0x30, 4, FieldType::U32Le),
            TemplateField::new("ELF Header Size", 0x34, 2, FieldType::U16Le),
            TemplateField::new("Phdr Entry Size", 0x36, 2, FieldType::U16Le),
            TemplateField::new("Phdr Entry Count", 0x38, 2, FieldType::U16Le),
            TemplateField::new("Shdr Entry Size", 0x3A, 2, FieldType::U16Le),
            TemplateField::new("Shdr Entry Count", 0x3C, 2, FieldType::U16Le),
            TemplateField::new("Shstr Section Index", 0x3E, 2, FieldType::U16Le),
        ],
    }
}

fn make_pe() -> FormatTemplate {
    FormatTemplate {
        name: "PE Executable".to_string(),
        magic: b"MZ".to_vec(),
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("DOS Magic", 0x00, 2, FieldType::Bytes),
            TemplateField::new("DOS Last Page Bytes", 0x02, 2, FieldType::U16Le),
            TemplateField::new("DOS Page Count", 0x04, 2, FieldType::U16Le),
            TemplateField::new("DOS Relocation Count", 0x06, 2, FieldType::U16Le),
            TemplateField::new("DOS Header Paragraphs", 0x08, 2, FieldType::U16Le),
            TemplateField::new("DOS Min Alloc Paragraphs", 0x0A, 2, FieldType::U16Le),
            TemplateField::new("DOS Max Alloc Paragraphs", 0x0C, 2, FieldType::U16Le),
            TemplateField::new("DOS Initial SS", 0x0E, 2, FieldType::U16Le),
            TemplateField::new("DOS Initial SP", 0x10, 2, FieldType::U16Le),
            TemplateField::new("DOS Checksum", 0x12, 2, FieldType::U16Le),
            TemplateField::new("DOS Initial IP", 0x14, 2, FieldType::U16Le),
            TemplateField::new("DOS Initial CS", 0x16, 2, FieldType::U16Le),
            TemplateField::new("DOS Relocation Table Offset", 0x18, 2, FieldType::U16Le),
            TemplateField::new("DOS Overlay Number", 0x1A, 2, FieldType::U16Le),
            TemplateField::new("PE Header Offset", 0x3C, 4, FieldType::U32Le),
        ],
    }
}

fn make_sqlite() -> FormatTemplate {
    FormatTemplate {
        name: "SQLite Database".to_string(),
        magic: b"SQLite format 3\x00".to_vec(),
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("SQLite Magic", 0x00, 16, FieldType::AsciiStr),
            TemplateField::new("Page Size", 0x10, 2, FieldType::U16Be),
            TemplateField::new("Write Format Version", 0x12, 1, FieldType::U8),
            TemplateField::new("Read Format Version", 0x13, 1, FieldType::U8),
            TemplateField::new("Reserved Bytes Per Page", 0x14, 1, FieldType::U8),
            TemplateField::new("Max Embedded Payload Fraction", 0x15, 1, FieldType::U8),
            TemplateField::new("Min Embedded Payload Fraction", 0x16, 1, FieldType::U8),
            TemplateField::new("Leaf Payload Fraction", 0x17, 1, FieldType::U8),
            TemplateField::new("File Change Counter", 0x18, 4, FieldType::U32Be),
            TemplateField::new("Database Page Count", 0x1C, 4, FieldType::U32Be),
            TemplateField::new("First Freelist Trunk Page", 0x20, 4, FieldType::U32Be),
            TemplateField::new("Total Freelist Pages", 0x24, 4, FieldType::U32Be),
            TemplateField::new("Schema Cookie", 0x28, 4, FieldType::U32Be),
            TemplateField::new("Schema Format Number", 0x2C, 4, FieldType::U32Be),
            TemplateField::new("Default Page Cache Size", 0x30, 4, FieldType::U32Be),
            TemplateField::new("Largest Root B-Tree Page", 0x34, 4, FieldType::U32Be),
            TemplateField::new("Text Encoding", 0x38, 4, FieldType::U32Be),
            TemplateField::new("User Version", 0x3C, 4, FieldType::U32Be),
            TemplateField::new("Incremental Vacuum Mode", 0x40, 4, FieldType::U32Be),
            TemplateField::new("Application ID", 0x44, 4, FieldType::U32Be),
            TemplateField::new("Version Valid For", 0x60, 4, FieldType::U32Be),
            TemplateField::new("SQLite Version Number", 0x64, 4, FieldType::U32Be),
        ],
    }
}

fn make_jpeg() -> FormatTemplate {
    FormatTemplate {
        name: "JPEG Image".to_string(),
        magic: vec![0xFF, 0xD8, 0xFF],
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("SOI Marker", 0x00, 2, FieldType::Bytes),
            TemplateField::new("APP0/APP1 Marker", 0x02, 2, FieldType::Bytes),
            TemplateField::new("Segment Length", 0x04, 2, FieldType::U16Be),
            TemplateField::new("Identifier", 0x06, 5, FieldType::AsciiStr),
        ],
    }
}

fn make_gif() -> FormatTemplate {
    FormatTemplate {
        name: "GIF Image".to_string(),
        magic: b"GIF8".to_vec(),
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("GIF Signature", 0x00, 6, FieldType::AsciiStr),
            TemplateField::new("Logical Screen Width", 0x06, 2, FieldType::U16Le),
            TemplateField::new("Logical Screen Height", 0x08, 2, FieldType::U16Le),
            TemplateField::new("Packed Field", 0x0A, 1, FieldType::U8),
            TemplateField::new("Background Color Index", 0x0B, 1, FieldType::U8),
            TemplateField::new("Pixel Aspect Ratio", 0x0C, 1, FieldType::U8),
        ],
    }
}

fn make_bmp() -> FormatTemplate {
    FormatTemplate {
        name: "BMP Image".to_string(),
        magic: b"BM".to_vec(),
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("BMP Magic", 0x00, 2, FieldType::Bytes),
            TemplateField::new("File Size", 0x02, 4, FieldType::U32Le),
            TemplateField::new("Reserved 1", 0x06, 2, FieldType::U16Le),
            TemplateField::new("Reserved 2", 0x08, 2, FieldType::U16Le),
            TemplateField::new("Pixel Data Offset", 0x0A, 4, FieldType::U32Le),
            TemplateField::new("DIB Header Size", 0x0E, 4, FieldType::U32Le),
            TemplateField::new("Image Width", 0x12, 4, FieldType::U32Le),
            TemplateField::new("Image Height", 0x16, 4, FieldType::U32Le),
            TemplateField::new("Color Planes", 0x1A, 2, FieldType::U16Le),
            TemplateField::new("Bits Per Pixel", 0x1C, 2, FieldType::U16Le),
            TemplateField::new("Compression Method", 0x1E, 4, FieldType::U32Le),
            TemplateField::new("Image Data Size", 0x22, 4, FieldType::U32Le),
            TemplateField::new("X Pixels Per Meter", 0x26, 4, FieldType::U32Le),
            TemplateField::new("Y Pixels Per Meter", 0x2A, 4, FieldType::U32Le),
            TemplateField::new("Colors In Table", 0x2E, 4, FieldType::U32Le),
            TemplateField::new("Important Color Count", 0x32, 4, FieldType::U32Le),
        ],
    }
}

fn make_wav() -> FormatTemplate {
    FormatTemplate {
        name: "WAV Audio".to_string(),
        magic: b"RIFF".to_vec(),
        magic_offset: 0,
        second_magic: Some((b"WAVE".to_vec(), 8)),
        fields: vec![
            TemplateField::new("RIFF Marker", 0x00, 4, FieldType::Bytes),
            TemplateField::new("File Size - 8", 0x04, 4, FieldType::U32Le),
            TemplateField::new("WAVE Marker", 0x08, 4, FieldType::Bytes),
            TemplateField::new("fmt Chunk Marker", 0x0C, 4, FieldType::Bytes),
            TemplateField::new("fmt Chunk Size", 0x10, 4, FieldType::U32Le),
            TemplateField::new("Audio Format", 0x14, 2, FieldType::U16Le),
            TemplateField::new("Number of Channels", 0x16, 2, FieldType::U16Le),
            TemplateField::new("Sample Rate", 0x18, 4, FieldType::U32Le),
            TemplateField::new("Byte Rate", 0x1C, 4, FieldType::U32Le),
            TemplateField::new("Block Align", 0x20, 2, FieldType::U16Le),
            TemplateField::new("Bits Per Sample", 0x22, 2, FieldType::U16Le),
            TemplateField::new("data Chunk Marker", 0x24, 4, FieldType::Bytes),
            TemplateField::new("data Chunk Size", 0x28, 4, FieldType::U32Le),
        ],
    }
}

fn make_pdf() -> FormatTemplate {
    FormatTemplate {
        name: "PDF Document".to_string(),
        magic: b"%PDF-".to_vec(),
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("PDF Signature", 0x00, 5, FieldType::Bytes),
            TemplateField::new("PDF Version", 0x05, 3, FieldType::AsciiStr),
        ],
    }
}

fn make_macho_le64() -> FormatTemplate {
    // MH_MAGIC_64 little-endian (modern macOS / x86_64 / arm64)
    FormatTemplate {
        name: "Mach-O Binary".to_string(),
        magic: vec![0xCF, 0xFA, 0xED, 0xFE],
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("Magic", 0x00, 4, FieldType::U32Le),
            TemplateField::new("CPU Type", 0x04, 4, FieldType::U32Le),
            TemplateField::new("CPU Subtype", 0x08, 4, FieldType::U32Le),
            TemplateField::new("File Type", 0x0C, 4, FieldType::U32Le),
            TemplateField::new("Number of Load Commands", 0x10, 4, FieldType::U32Le),
            TemplateField::new("Load Commands Size", 0x14, 4, FieldType::U32Le),
            TemplateField::new("Flags", 0x18, 4, FieldType::U32Le),
            TemplateField::new("Reserved", 0x1C, 4, FieldType::U32Le),
        ],
    }
}

fn make_macho_le32() -> FormatTemplate {
    // MH_CIGAM little-endian 32-bit
    FormatTemplate {
        name: "Mach-O Binary (32-bit)".to_string(),
        magic: vec![0xCE, 0xFA, 0xED, 0xFE],
        magic_offset: 0,
        second_magic: None,
        fields: vec![
            TemplateField::new("Magic", 0x00, 4, FieldType::U32Le),
            TemplateField::new("CPU Type", 0x04, 4, FieldType::U32Le),
            TemplateField::new("CPU Subtype", 0x08, 4, FieldType::U32Le),
            TemplateField::new("File Type", 0x0C, 4, FieldType::U32Le),
            TemplateField::new("Number of Load Commands", 0x10, 4, FieldType::U32Le),
            TemplateField::new("Load Commands Size", 0x14, 4, FieldType::U32Le),
            TemplateField::new("Flags", 0x18, 4, FieldType::U32Le),
        ],
    }
}

// ─── Registry ────────────────────────────────────────────────────────────────

/// Returns all built-in format templates.
pub fn builtin_templates() -> Vec<FormatTemplate> {
    vec![
        make_png(),
        make_zip(),
        make_elf(),
        make_pe(),
        make_sqlite(),
        make_jpeg(),
        make_gif(),
        make_bmp(),
        make_wav(),
        make_pdf(),
        make_macho_le64(),
        make_macho_le32(),
    ]
}

/// Detect the format of `data` by checking magic bytes against all known
/// templates (built-ins first, then `extra`). Returns the first match.
pub fn detect_format(data: &[u8], extra: &[FormatTemplate]) -> Option<FormatTemplate> {
    for tmpl in builtin_templates().iter().chain(extra.iter()) {
        if tmpl.matches(data) {
            return Some(tmpl.clone());
        }
    }
    None
}

// ─── TOML custom template loading ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct TomlField {
    name: String,
    offset: usize,
    size: usize,
    #[serde(default = "default_field_type")]
    field_type: String,
}

fn default_field_type() -> String {
    "bytes".to_string()
}

#[derive(Debug, Deserialize)]
struct TomlTemplate {
    name: String,
    #[serde(default)]
    magic: Vec<u8>,
    #[serde(default)]
    magic_offset: usize,
    #[serde(default)]
    fields: Vec<TomlField>,
}

impl From<TomlTemplate> for FormatTemplate {
    fn from(t: TomlTemplate) -> Self {
        FormatTemplate {
            name: t.name,
            magic: t.magic,
            magic_offset: t.magic_offset,
            second_magic: None,
            fields: t
                .fields
                .into_iter()
                .map(|f| TemplateField::new(&f.name, f.offset, f.size, FieldType::from_str(&f.field_type)))
                .collect(),
        }
    }
}

/// Parse a `FormatTemplate` from a TOML string.
pub fn parse_toml_template(toml_str: &str) -> Result<FormatTemplate, String> {
    let t: TomlTemplate = toml::from_str(toml_str).map_err(|e| e.to_string())?;
    Ok(t.into())
}

/// Load user-defined templates from `~/.config/chx/templates/*.toml`.
/// Silently skips files that fail to parse.
pub fn load_custom_templates() -> Vec<FormatTemplate> {
    let mut templates = Vec::new();
    let config_dir = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".config").join("chx").join("templates"))
        .unwrap_or_default();

    if !config_dir.exists() {
        return templates;
    }

    if let Ok(entries) = std::fs::read_dir(&config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match parse_toml_template(&content) {
                        Ok(tmpl) => templates.push(tmpl),
                        Err(e) => eprintln!("chx: failed to parse template {:?}: {}", path, e),
                    }
                }
            }
        }
    }
    templates
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_magic_matches() {
        let magic = [0x89u8, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let tmpl = make_png();
        assert!(tmpl.matches(&magic));
    }

    #[test]
    fn png_magic_no_match_on_wrong_bytes() {
        let data = [0x89u8, 0x50, 0x4E, 0x47, 0x00, 0x00, 0x00, 0x00];
        let tmpl = make_png();
        assert!(!tmpl.matches(&data));
    }

    #[test]
    fn zip_magic_matches() {
        let data = b"PK\x03\x04extra";
        let tmpl = make_zip();
        assert!(tmpl.matches(data));
    }

    #[test]
    fn elf_magic_matches() {
        let data = b"\x7fELF\x02\x01\x01\x00";
        let tmpl = make_elf();
        assert!(tmpl.matches(data));
    }

    #[test]
    fn pe_magic_matches() {
        let data = b"MZextra";
        let tmpl = make_pe();
        assert!(tmpl.matches(data));
    }

    #[test]
    fn sqlite_magic_matches() {
        let magic = b"SQLite format 3\x00extra";
        let tmpl = make_sqlite();
        assert!(tmpl.matches(magic));
    }

    #[test]
    fn wav_requires_wave_marker() {
        let mut data = [0u8; 16];
        data[0..4].copy_from_slice(b"RIFF");
        let tmpl = make_wav();
        // Missing WAVE at offset 8 — should not match
        assert!(!tmpl.matches(&data));
        // Now set WAVE at offset 8
        data[8..12].copy_from_slice(b"WAVE");
        assert!(tmpl.matches(&data));
    }

    #[test]
    fn detect_png() {
        let magic = [0x89u8, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let result = detect_format(&magic, &[]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "PNG Image");
    }

    #[test]
    fn detect_unknown_returns_none() {
        let data = [0x00u8; 16];
        assert!(detect_format(&data, &[]).is_none());
    }

    #[test]
    fn parse_u32le_field() {
        let field = TemplateField::new("Test", 0, 4, FieldType::U32Le);
        let bytes = [0x78u8, 0x56, 0x34, 0x12];
        let result = parse_field_value(&field, &bytes);
        assert!(result.contains("0x12345678"));
    }

    #[test]
    fn parse_u32be_field() {
        let field = TemplateField::new("Test", 0, 4, FieldType::U32Be);
        let bytes = [0x12u8, 0x34, 0x56, 0x78];
        let result = parse_field_value(&field, &bytes);
        assert!(result.contains("0x12345678"));
    }

    #[test]
    fn parse_u16le_field() {
        let field = TemplateField::new("Test", 0, 2, FieldType::U16Le);
        let bytes = [0x01u8, 0x00];
        let result = parse_field_value(&field, &bytes);
        assert!(result.contains("1"));
    }

    #[test]
    fn parse_ascii_field() {
        let field = TemplateField::new("Test", 0, 5, FieldType::AsciiStr);
        let bytes = b"Hello";
        let result = parse_field_value(&field, bytes);
        assert!(result.contains("Hello"));
    }

    #[test]
    fn parse_bytes_field() {
        let field = TemplateField::new("Test", 0, 4, FieldType::Bytes);
        let bytes = [0xDEu8, 0xAD, 0xBE, 0xEF];
        let result = parse_field_value(&field, &bytes);
        assert!(result.contains("DE"));
        assert!(result.contains("EF"));
    }

    #[test]
    fn parse_out_of_range() {
        let field = TemplateField::new("Test", 0, 4, FieldType::U32Le);
        let result = parse_field_value(&field, &[0x01, 0x02]); // only 2 bytes
        assert!(result.contains("out of range"));
    }

    #[test]
    fn build_field_map_covers_all_bytes() {
        let fields = vec![
            TemplateField::new("A", 0, 4, FieldType::Bytes),
            TemplateField::new("B", 4, 2, FieldType::U16Le),
        ];
        let map = build_field_map(&fields);
        // All 6 bytes should be in the map
        for i in 0..6 {
            assert!(map.contains_key(&i), "byte {} missing from map", i);
        }
        assert!(!map.contains_key(&6));
    }

    #[test]
    fn toml_template_parsing() {
        let toml = r#"
name = "Test Format"
magic = [0xDE, 0xAD, 0xBE, 0xEF]
magic_offset = 0

[[fields]]
name = "Header"
offset = 0
size = 4
field_type = "bytes"

[[fields]]
name = "Version"
offset = 4
size = 2
field_type = "u16le"
"#;
        let tmpl = parse_toml_template(toml).unwrap();
        assert_eq!(tmpl.name, "Test Format");
        assert_eq!(tmpl.magic, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(tmpl.fields.len(), 2);
        assert_eq!(tmpl.fields[0].name, "Header");
        assert_eq!(tmpl.fields[1].field_type, FieldType::U16Le);
    }

    #[test]
    fn toml_template_invalid_fails() {
        let result = parse_toml_template("not valid toml !!!@@@");
        assert!(result.is_err());
    }

    #[test]
    fn builtin_templates_count() {
        assert!(builtin_templates().len() >= 5, "need at least 5 built-in templates");
    }

    #[test]
    fn all_builtin_templates_have_fields() {
        for tmpl in builtin_templates() {
            assert!(!tmpl.fields.is_empty(), "{} has no fields", tmpl.name);
        }
    }

    #[test]
    fn png_resolve_fields_static_only_when_no_chunks() {
        let magic = [0x89u8, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let tmpl = make_png();
        let fields = tmpl.resolve_fields(&magic);
        // At minimum the static fields should be present
        assert!(!fields.is_empty());
    }

    #[test]
    fn field_type_from_str_roundtrip() {
        assert_eq!(FieldType::from_str("u8"), FieldType::U8);
        assert_eq!(FieldType::from_str("u16le"), FieldType::U16Le);
        assert_eq!(FieldType::from_str("u16be"), FieldType::U16Be);
        assert_eq!(FieldType::from_str("u32le"), FieldType::U32Le);
        assert_eq!(FieldType::from_str("u32be"), FieldType::U32Be);
        assert_eq!(FieldType::from_str("u64le"), FieldType::U64Le);
        assert_eq!(FieldType::from_str("u64be"), FieldType::U64Be);
        assert_eq!(FieldType::from_str("ascii"), FieldType::AsciiStr);
        assert_eq!(FieldType::from_str("bytes"), FieldType::Bytes);
        assert_eq!(FieldType::from_str("unknown"), FieldType::Bytes);
    }
}
