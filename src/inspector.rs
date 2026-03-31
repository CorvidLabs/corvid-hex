/// Data inspector: interprets bytes at cursor position as various data types.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    U8,
    I8,
    Binary,
    Octal,
    Ascii,
    Utf8,
    U16Le,
    U16Be,
    I16Le,
    I16Be,
    U32Le,
    U32Be,
    I32Le,
    I32Be,
    F32Le,
    F32Be,
    U64Le,
    U64Be,
    I64Le,
    I64Be,
    F64Le,
    F64Be,
}

impl FieldType {
    pub fn byte_count(self) -> usize {
        match self {
            FieldType::U8
            | FieldType::I8
            | FieldType::Binary
            | FieldType::Octal
            | FieldType::Ascii
            | FieldType::Utf8 => 1,
            FieldType::U16Le
            | FieldType::U16Be
            | FieldType::I16Le
            | FieldType::I16Be => 2,
            FieldType::U32Le
            | FieldType::U32Be
            | FieldType::I32Le
            | FieldType::I32Be
            | FieldType::F32Le
            | FieldType::F32Be => 4,
            FieldType::U64Le
            | FieldType::U64Be
            | FieldType::I64Le
            | FieldType::I64Be
            | FieldType::F64Le
            | FieldType::F64Be => 8,
        }
    }

    pub fn is_editable(self) -> bool {
        !matches!(
            self,
            FieldType::Binary | FieldType::Octal | FieldType::Ascii | FieldType::Utf8
        )
    }

    /// Parse a string as this field type and return the bytes to write, or None if invalid.
    pub fn parse(self, input: &str) -> Option<Vec<u8>> {
        let s = input.trim();
        match self {
            FieldType::U8 => {
                let v: u8 = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                    u8::from_str_radix(hex, 16).ok()?
                } else {
                    s.parse().ok()?
                };
                Some(vec![v])
            }
            FieldType::I8 => {
                let v: i8 = s.parse().ok()?;
                Some(vec![v as u8])
            }
            FieldType::U16Le => Some(parse_u16(s)?.to_le_bytes().to_vec()),
            FieldType::U16Be => Some(parse_u16(s)?.to_be_bytes().to_vec()),
            FieldType::I16Le => {
                let v: i16 = s.parse().ok()?;
                Some(v.to_le_bytes().to_vec())
            }
            FieldType::I16Be => {
                let v: i16 = s.parse().ok()?;
                Some(v.to_be_bytes().to_vec())
            }
            FieldType::U32Le => Some(parse_u32(s)?.to_le_bytes().to_vec()),
            FieldType::U32Be => Some(parse_u32(s)?.to_be_bytes().to_vec()),
            FieldType::I32Le => {
                let v: i32 = s.parse().ok()?;
                Some(v.to_le_bytes().to_vec())
            }
            FieldType::I32Be => {
                let v: i32 = s.parse().ok()?;
                Some(v.to_be_bytes().to_vec())
            }
            FieldType::F32Le => {
                let v: f32 = s.parse().ok()?;
                Some(v.to_le_bytes().to_vec())
            }
            FieldType::F32Be => {
                let v: f32 = s.parse().ok()?;
                Some(v.to_be_bytes().to_vec())
            }
            FieldType::U64Le => Some(parse_u64(s)?.to_le_bytes().to_vec()),
            FieldType::U64Be => Some(parse_u64(s)?.to_be_bytes().to_vec()),
            FieldType::I64Le => {
                let v: i64 = s.parse().ok()?;
                Some(v.to_le_bytes().to_vec())
            }
            FieldType::I64Be => {
                let v: i64 = s.parse().ok()?;
                Some(v.to_be_bytes().to_vec())
            }
            FieldType::F64Le => {
                let v: f64 = s.parse().ok()?;
                Some(v.to_le_bytes().to_vec())
            }
            FieldType::F64Be => {
                let v: f64 = s.parse().ok()?;
                Some(v.to_be_bytes().to_vec())
            }
            FieldType::Binary | FieldType::Octal | FieldType::Ascii | FieldType::Utf8 => None,
        }
    }
}

fn parse_u16(s: &str) -> Option<u16> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u16::from_str_radix(hex, 16).ok()
    } else {
        s.parse().ok()
    }
}

fn parse_u32(s: &str) -> Option<u32> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        s.parse().ok()
    }
}

fn parse_u64(s: &str) -> Option<u64> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        s.parse().ok()
    }
}

#[derive(Debug, Clone)]
pub struct InspectorField {
    pub label: &'static str,
    pub value: String,
    pub field_type: FieldType,
}

/// Interpret up to 8 bytes starting at the cursor as various data types.
/// Returns all applicable fields based on how many bytes are available.
pub fn interpret(bytes: &[u8]) -> Vec<InspectorField> {
    let mut fields = Vec::new();

    if let Some(&b) = bytes.first() {
        fields.push(InspectorField {
            label: "u8",
            value: b.to_string(),
            field_type: FieldType::U8,
        });
        fields.push(InspectorField {
            label: "i8",
            value: (b as i8).to_string(),
            field_type: FieldType::I8,
        });
        fields.push(InspectorField {
            label: "bin",
            value: format!("{:08b}", b),
            field_type: FieldType::Binary,
        });
        fields.push(InspectorField {
            label: "oct",
            value: format!("{:o}", b),
            field_type: FieldType::Octal,
        });

        let ascii_val = if b.is_ascii_graphic() || b == b' ' {
            format!("'{}'", b as char)
        } else {
            format!("0x{:02X}", b)
        };
        fields.push(InspectorField {
            label: "ascii",
            value: ascii_val,
            field_type: FieldType::Ascii,
        });

        // UTF-8: try 1-4 bytes, longest valid sequence first
        let utf8_limit = bytes.len().min(4);
        let utf8_val = {
            let mut result = String::new();
            for len in (1..=utf8_limit).rev() {
                if let Ok(s) = std::str::from_utf8(&bytes[..len]) {
                    if let Some(c) = s.chars().next() {
                        if !c.is_control() {
                            result = if len == 1 {
                                format!("'{}'", c)
                            } else {
                                format!("'{}' ({}B)", c, len)
                            };
                            break;
                        }
                    }
                }
            }
            if result.is_empty() {
                format!("0x{:02X}", b)
            } else {
                result
            }
        };
        fields.push(InspectorField {
            label: "utf-8",
            value: utf8_val,
            field_type: FieldType::Utf8,
        });
    }

    if bytes.len() >= 2 {
        let arr2 = [bytes[0], bytes[1]];
        fields.push(InspectorField {
            label: "u16 LE",
            value: u16::from_le_bytes(arr2).to_string(),
            field_type: FieldType::U16Le,
        });
        fields.push(InspectorField {
            label: "u16 BE",
            value: u16::from_be_bytes(arr2).to_string(),
            field_type: FieldType::U16Be,
        });
        fields.push(InspectorField {
            label: "i16 LE",
            value: i16::from_le_bytes(arr2).to_string(),
            field_type: FieldType::I16Le,
        });
        fields.push(InspectorField {
            label: "i16 BE",
            value: i16::from_be_bytes(arr2).to_string(),
            field_type: FieldType::I16Be,
        });
    }

    if bytes.len() >= 4 {
        let arr4 = [bytes[0], bytes[1], bytes[2], bytes[3]];
        fields.push(InspectorField {
            label: "u32 LE",
            value: u32::from_le_bytes(arr4).to_string(),
            field_type: FieldType::U32Le,
        });
        fields.push(InspectorField {
            label: "u32 BE",
            value: u32::from_be_bytes(arr4).to_string(),
            field_type: FieldType::U32Be,
        });
        fields.push(InspectorField {
            label: "i32 LE",
            value: i32::from_le_bytes(arr4).to_string(),
            field_type: FieldType::I32Le,
        });
        fields.push(InspectorField {
            label: "i32 BE",
            value: i32::from_be_bytes(arr4).to_string(),
            field_type: FieldType::I32Be,
        });
        fields.push(InspectorField {
            label: "f32 LE",
            value: format!("{:.4e}", f32::from_le_bytes(arr4)),
            field_type: FieldType::F32Le,
        });
        fields.push(InspectorField {
            label: "f32 BE",
            value: format!("{:.4e}", f32::from_be_bytes(arr4)),
            field_type: FieldType::F32Be,
        });
    }

    if bytes.len() >= 8 {
        let arr8: [u8; 8] = bytes[..8].try_into().unwrap();
        fields.push(InspectorField {
            label: "u64 LE",
            value: u64::from_le_bytes(arr8).to_string(),
            field_type: FieldType::U64Le,
        });
        fields.push(InspectorField {
            label: "u64 BE",
            value: u64::from_be_bytes(arr8).to_string(),
            field_type: FieldType::U64Be,
        });
        fields.push(InspectorField {
            label: "i64 LE",
            value: i64::from_le_bytes(arr8).to_string(),
            field_type: FieldType::I64Le,
        });
        fields.push(InspectorField {
            label: "i64 BE",
            value: i64::from_be_bytes(arr8).to_string(),
            field_type: FieldType::I64Be,
        });
        fields.push(InspectorField {
            label: "f64 LE",
            value: format!("{:.4e}", f64::from_le_bytes(arr8)),
            field_type: FieldType::F64Le,
        });
        fields.push(InspectorField {
            label: "f64 BE",
            value: format!("{:.4e}", f64::from_be_bytes(arr8)),
            field_type: FieldType::F64Be,
        });
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_i8_all_ones() {
        let bytes = [0xFFu8];
        let fields = interpret(&bytes);
        let u8f = fields.iter().find(|f| f.label == "u8").unwrap();
        assert_eq!(u8f.value, "255");
        let i8f = fields.iter().find(|f| f.label == "i8").unwrap();
        assert_eq!(i8f.value, "-1");
    }

    #[test]
    fn test_binary_representation() {
        let bytes = [0xA5u8];
        let fields = interpret(&bytes);
        let bin = fields.iter().find(|f| f.label == "bin").unwrap();
        assert_eq!(bin.value, "10100101");
    }

    #[test]
    fn test_octal_representation() {
        let bytes = [0o177u8]; // 127
        let fields = interpret(&bytes);
        let oct = fields.iter().find(|f| f.label == "oct").unwrap();
        assert_eq!(oct.value, "177");
    }

    #[test]
    fn test_u16_le_be() {
        let bytes = [0x01u8, 0x02];
        let fields = interpret(&bytes);
        let u16le = fields.iter().find(|f| f.label == "u16 LE").unwrap();
        assert_eq!(u16le.value, "513"); // 0x0201 LE = 513
        let u16be = fields.iter().find(|f| f.label == "u16 BE").unwrap();
        assert_eq!(u16be.value, "258"); // 0x0102 BE = 258
    }

    #[test]
    fn test_i16_le_be() {
        let bytes = [0xFFu8, 0xFF];
        let fields = interpret(&bytes);
        let i16le = fields.iter().find(|f| f.label == "i16 LE").unwrap();
        assert_eq!(i16le.value, "-1");
        let i16be = fields.iter().find(|f| f.label == "i16 BE").unwrap();
        assert_eq!(i16be.value, "-1");
    }

    #[test]
    fn test_u32_le_be() {
        let bytes = [0x01u8, 0x02, 0x03, 0x04];
        let fields = interpret(&bytes);
        let u32le = fields.iter().find(|f| f.label == "u32 LE").unwrap();
        assert_eq!(u32le.value, "67305985"); // 0x04030201
        let u32be = fields.iter().find(|f| f.label == "u32 BE").unwrap();
        assert_eq!(u32be.value, "16909060"); // 0x01020304
    }

    #[test]
    fn test_i32_all_ones() {
        let bytes = [0xFFu8, 0xFF, 0xFF, 0xFF];
        let fields = interpret(&bytes);
        let i32le = fields.iter().find(|f| f.label == "i32 LE").unwrap();
        assert_eq!(i32le.value, "-1");
        let i32be = fields.iter().find(|f| f.label == "i32 BE").unwrap();
        assert_eq!(i32be.value, "-1");
    }

    #[test]
    fn test_u64_le_be() {
        let bytes = [0x01u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let fields = interpret(&bytes);
        let u64le = fields.iter().find(|f| f.label == "u64 LE").unwrap();
        assert_eq!(u64le.value, "1");
        let u64be = fields.iter().find(|f| f.label == "u64 BE").unwrap();
        assert_eq!(u64be.value, "72057594037927936"); // 0x0100000000000000
    }

    #[test]
    fn test_i64_all_ones() {
        let bytes = [0xFFu8; 8];
        let fields = interpret(&bytes);
        let i64le = fields.iter().find(|f| f.label == "i64 LE").unwrap();
        assert_eq!(i64le.value, "-1");
    }

    #[test]
    fn test_ascii_printable() {
        let bytes = [b'A'];
        let fields = interpret(&bytes);
        let ascii = fields.iter().find(|f| f.label == "ascii").unwrap();
        assert_eq!(ascii.value, "'A'");
    }

    #[test]
    fn test_ascii_non_printable() {
        let bytes = [0x01u8];
        let fields = interpret(&bytes);
        let ascii = fields.iter().find(|f| f.label == "ascii").unwrap();
        assert!(ascii.value.starts_with("0x"), "expected hex repr, got: {}", ascii.value);
    }

    #[test]
    fn test_empty_bytes_produces_no_fields() {
        let fields = interpret(&[]);
        assert!(fields.is_empty());
    }

    #[test]
    fn test_single_byte_no_multibyte_fields() {
        let bytes = [0x41u8];
        let fields = interpret(&bytes);
        assert!(!fields.iter().any(|f| f.label.contains("u16")));
        assert!(!fields.iter().any(|f| f.label.contains("u32")));
        assert!(!fields.iter().any(|f| f.label.contains("u64")));
        assert!(!fields.iter().any(|f| f.label.contains("f32")));
        assert!(!fields.iter().any(|f| f.label.contains("f64")));
    }

    #[test]
    fn test_two_bytes_no_32bit_fields() {
        let bytes = [0x01u8, 0x02];
        let fields = interpret(&bytes);
        assert!(fields.iter().any(|f| f.label.contains("u16")));
        assert!(!fields.iter().any(|f| f.label.contains("u32")));
    }

    #[test]
    fn test_parse_u8_decimal() {
        assert_eq!(FieldType::U8.parse("42"), Some(vec![42]));
        assert_eq!(FieldType::U8.parse("255"), Some(vec![255]));
        assert_eq!(FieldType::U8.parse("256"), None);
        assert_eq!(FieldType::U8.parse("-1"), None);
    }

    #[test]
    fn test_parse_u8_hex() {
        assert_eq!(FieldType::U8.parse("0xFF"), Some(vec![255]));
        assert_eq!(FieldType::U8.parse("0x41"), Some(vec![0x41]));
    }

    #[test]
    fn test_parse_i8() {
        assert_eq!(FieldType::I8.parse("-1"), Some(vec![0xFF]));
        assert_eq!(FieldType::I8.parse("127"), Some(vec![127]));
        assert_eq!(FieldType::I8.parse("-128"), Some(vec![128]));
        assert_eq!(FieldType::I8.parse("200"), None);
    }

    #[test]
    fn test_parse_u16_le_be() {
        assert_eq!(FieldType::U16Le.parse("256"), Some(vec![0x00, 0x01]));
        assert_eq!(FieldType::U16Be.parse("256"), Some(vec![0x01, 0x00]));
    }

    #[test]
    fn test_parse_u16_hex() {
        assert_eq!(FieldType::U16Le.parse("0x0102"), Some(vec![0x02, 0x01]));
        assert_eq!(FieldType::U16Be.parse("0x0102"), Some(vec![0x01, 0x02]));
    }

    #[test]
    fn test_parse_i16() {
        assert_eq!(FieldType::I16Le.parse("-1"), Some(vec![0xFF, 0xFF]));
        assert_eq!(FieldType::I16Be.parse("-1"), Some(vec![0xFF, 0xFF]));
    }

    #[test]
    fn test_parse_u32_le_be() {
        let le = FieldType::U32Le.parse("16909060").unwrap(); // 0x01020304
        assert_eq!(le, vec![0x04, 0x03, 0x02, 0x01]);
        let be = FieldType::U32Be.parse("16909060").unwrap();
        assert_eq!(be, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_parse_i32() {
        let bytes = FieldType::I32Le.parse("-1").unwrap();
        assert_eq!(bytes, vec![0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn test_parse_f32() {
        let bytes = FieldType::F32Le.parse("0.0").unwrap();
        assert_eq!(bytes, 0.0f32.to_le_bytes().to_vec());
        assert!(FieldType::F32Le.parse("notanumber").is_none());
    }

    #[test]
    fn test_parse_u64_le_be() {
        assert_eq!(FieldType::U64Le.parse("1"), Some(vec![1, 0, 0, 0, 0, 0, 0, 0]));
        assert_eq!(FieldType::U64Be.parse("1"), Some(vec![0, 0, 0, 0, 0, 0, 0, 1]));
    }

    #[test]
    fn test_parse_i64() {
        let bytes = FieldType::I64Le.parse("-1").unwrap();
        assert_eq!(bytes, vec![0xFF; 8]);
    }

    #[test]
    fn test_parse_f64() {
        let bytes = FieldType::F64Le.parse("0.0").unwrap();
        assert_eq!(bytes, 0.0f64.to_le_bytes().to_vec());
    }

    #[test]
    fn test_non_editable_fields_return_none_on_parse() {
        assert_eq!(FieldType::Binary.parse("10101010"), None);
        assert_eq!(FieldType::Octal.parse("177"), None);
        assert_eq!(FieldType::Ascii.parse("A"), None);
        assert_eq!(FieldType::Utf8.parse("hello"), None);
    }

    #[test]
    fn test_is_editable() {
        assert!(!FieldType::Binary.is_editable());
        assert!(!FieldType::Octal.is_editable());
        assert!(!FieldType::Ascii.is_editable());
        assert!(!FieldType::Utf8.is_editable());
        assert!(FieldType::U8.is_editable());
        assert!(FieldType::I8.is_editable());
        assert!(FieldType::U16Le.is_editable());
        assert!(FieldType::I64Le.is_editable());
        assert!(FieldType::F32Be.is_editable());
        assert!(FieldType::F64Le.is_editable());
    }

    #[test]
    fn test_byte_counts() {
        assert_eq!(FieldType::U8.byte_count(), 1);
        assert_eq!(FieldType::I8.byte_count(), 1);
        assert_eq!(FieldType::Binary.byte_count(), 1);
        assert_eq!(FieldType::Utf8.byte_count(), 1);
        assert_eq!(FieldType::U16Le.byte_count(), 2);
        assert_eq!(FieldType::I16Be.byte_count(), 2);
        assert_eq!(FieldType::U32Le.byte_count(), 4);
        assert_eq!(FieldType::F32Be.byte_count(), 4);
        assert_eq!(FieldType::U64Le.byte_count(), 8);
        assert_eq!(FieldType::F64Be.byte_count(), 8);
    }

    #[test]
    fn test_utf8_multibyte() {
        // UTF-8 encoding of '€' is 3 bytes: 0xE2, 0x82, 0xAC
        let bytes = [0xE2u8, 0x82, 0xAC, 0x00];
        let fields = interpret(&bytes);
        let utf8 = fields.iter().find(|f| f.label == "utf-8").unwrap();
        assert!(utf8.value.contains('€'), "expected € in utf-8 value, got: {}", utf8.value);
    }
}
