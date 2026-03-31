---
module: inspector
version: 1
status: draft
files:
  - src/inspector.rs
db_tables: []
depends_on: []
---

## Purpose

Interprets raw bytes at the cursor position as various numeric and text data types. Provides a structured list of typed fields (u8, i8, binary, octal, ASCII, UTF-8, u16/i16/u32/i32/f32/u64/i64/f64 in both little-endian and big-endian) along with parsing support for editing field values back into bytes.

## Public API

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `FieldType` | `pub enum FieldType` | Enumerates all supported data interpretation types (U8, I8, Binary, Octal, Ascii, Utf8, U16Le, U16Be, I16Le, I16Be, U32Le, U32Be, I32Le, I32Be, F32Le, F32Be, U64Le, U64Be, I64Le, I64Be, F64Le, F64Be). |
| `byte_count` | `pub fn byte_count(self) -> usize` | Returns the number of bytes this field type requires (1, 2, 4, or 8). |
| `is_editable` | `pub fn is_editable(self) -> bool` | Returns true if the field type supports user editing. Binary, Octal, Ascii, and Utf8 are read-only. |
| `parse` | `pub fn parse(self, input: &str) -> Option<Vec<u8>>` | Parses a user-entered string as this field type and returns the byte representation, or `None` if invalid. Supports hex prefix (`0x`) for unsigned integer types. |
| `InspectorField` | `pub struct InspectorField` | A single interpreted field with `label: &'static str`, `value: String`, and `field_type: FieldType`. |
| `interpret` | `pub fn interpret(bytes: &[u8]) -> Vec<InspectorField>` | Interprets up to 8 bytes as all applicable data types, returning fields for each type that has enough bytes available. |

## Invariants

1. `interpret` with an empty slice returns an empty vec.
2. `interpret` only includes multi-byte fields when enough bytes are available (e.g., u16 requires >= 2 bytes).
3. `parse` returns `None` for non-editable field types (Binary, Octal, Ascii, Utf8).
4. `parse` output byte count always matches `byte_count` for editable types on valid input.
5. `byte_count` returns 1 for single-byte types, 2 for 16-bit, 4 for 32-bit, 8 for 64-bit.

## Behavioral Examples

**Single byte interpretation**
- Given: bytes = `[0x41]`
- When: `interpret` is called
- Then: returns fields for u8 (65), i8 (65), bin (01000001), oct (101), ascii ('A'), utf-8 ('A') — no 16/32/64-bit fields

**Multi-byte interpretation**
- Given: bytes = `[0x01, 0x02]`
- When: `interpret` is called
- Then: includes u16 LE (513) and u16 BE (258) fields, but no 32/64-bit fields

**Parse with hex prefix**
- Given: field type U8
- When: `parse("0xFF")` is called
- Then: returns `Some(vec![255])`

**Parse non-editable type**
- Given: field type Binary
- When: `parse("10101010")` is called
- Then: returns `None`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Empty byte slice to `interpret` | Returns empty vec |
| `parse` with invalid number | Returns `None` |
| `parse` with value out of range (e.g., 256 for U8) | Returns `None` |
| `parse` on non-editable field type | Returns `None` |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `std::str::from_utf8` | UTF-8 character decoding |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-30 | Initial spec |
