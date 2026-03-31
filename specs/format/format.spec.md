---
module: format
version: 1
status: draft
files:
  - src/format.rs
db_tables: []
depends_on: []
---

## Purpose

Provides a template system for parsing and labeling known binary file formats. Templates define named fields at specific byte offsets with typed interpretation (integers of various widths/endianness, ASCII strings, raw bytes). Ships with built-in templates for common formats (PNG, ZIP, ELF, PE, Mach-O, SQLite, JPEG, GIF, BMP, WAV, PDF) and supports user-defined TOML templates loaded from `~/.config/chx/templates/`.

## Public API

| Symbol | Signature | Description |
|--------|-----------|-------------|
| `FieldType` | `pub enum FieldType` | How a template field's bytes are interpreted. Variants: `U8`, `U16Le`, `U16Be`, `U32Le`, `U32Be`, `U64Le`, `U64Be`, `AsciiStr`, `Bytes`. |
| `from_str` | `pub fn from_str(s: &str) -> Self` | Parses a field type string (e.g. `"u16le"`, `"ascii"`) into a `FieldType`. Case-insensitive. Unknown strings default to `Bytes`. |
| `TemplateField` | `pub struct TemplateField` | A single named field: `name`, `offset`, `size`, `field_type`. |
| `parse_field_value` | `pub fn parse_field_value(field: &TemplateField, bytes: &[u8]) -> String` | Formats a field's bytes as a human-readable string based on its `FieldType`. Returns `"(out of range)"` if bytes are too short. |
| `FormatTemplate` | `pub struct FormatTemplate` | A complete format template: `name`, `magic`, `magic_offset`, `second_magic`, `fields`. |
| `matches` | `pub fn matches(&self, data: &[u8]) -> bool` | Checks if data matches this template's magic bytes (and optional secondary magic). |
| `resolve_fields` | `pub fn resolve_fields(&self, data: &[u8]) -> Vec<TemplateField>` | Returns static fields plus dynamically resolved chunk fields (PNG chunks, ZIP entries). |
| `build_field_map` | `pub fn build_field_map(fields: &[TemplateField]) -> HashMap<usize, (String, usize)>` | Builds a byte-offset to (field_name, field_index) lookup map covering every byte within each field's range. |
| `builtin_templates` | `pub fn builtin_templates() -> Vec<FormatTemplate>` | Returns all built-in format templates (PNG, ZIP, ELF, PE, Mach-O, SQLite, JPEG, GIF, BMP, WAV, PDF). |
| `detect_format` | `pub fn detect_format(data: &[u8], extra: &[FormatTemplate]) -> Option<FormatTemplate>` | Auto-detects a file's format by trying each template's magic bytes. Checks user-provided templates first, then built-ins. |
| `parse_toml_template` | `pub fn parse_toml_template(toml_str: &str) -> Result<FormatTemplate, String>` | Parses a TOML string into a `FormatTemplate`. |
| `load_custom_templates` | `pub fn load_custom_templates() -> Vec<FormatTemplate>` | Loads all `.toml` files from `~/.config/chx/templates/` as custom format templates. Silently skips unparseable files. |

## Invariants

1. `FieldType::from_str` is case-insensitive and defaults to `Bytes` for unknown strings.
2. `FormatTemplate::matches` returns false if magic bytes are empty or data is too short.
3. Secondary magic check is only performed if primary magic matches first.
4. `resolve_fields` only performs dynamic chunk resolution for "PNG Image" and "ZIP Archive" templates; all others return static fields only.
5. `build_field_map` uses first-wins semantics: if fields overlap at a byte offset, the first field in the list claims that offset.
6. `detect_format` checks user-provided (`extra`) templates before built-ins, allowing user overrides.
7. PNG chunk resolution walks chunks sequentially from offset 8 and stops when data is exhausted or a chunk would exceed bounds.
8. ZIP entry resolution walks local file headers starting at offset 0, stopping when the magic `PK\x03\x04` is not found.
9. Custom TOML templates require `name`, `magic`, and `magic_offset` fields; `fields` array and `second_magic` are optional.
10. `load_custom_templates` reads from `~/.config/chx/templates/` and silently ignores files that fail to parse.

## Behavioral Examples

**Auto-detect PNG**
- Given: data starts with `[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]`
- When: `detect_format(data, &[])` is called
- Then: returns the PNG template with IHDR fields and dynamic chunk resolution

**Parse u16le field**
- Given: a `TemplateField` with `field_type: U16Le`, `size: 2`, and bytes `[0x01, 0x00]`
- When: `parse_field_value` is called
- Then: returns `"1 (0x0001)"`

**Custom template override**
- Given: a custom template with the same magic as PNG is passed in `extra`
- When: `detect_format` is called on PNG data
- Then: returns the custom template (checked before built-ins)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Data shorter than magic offset + magic length | `matches` returns false |
| Field bytes shorter than field size | `parse_field_value` returns `"(out of range)"` |
| Invalid TOML in custom template file | `parse_toml_template` returns `Err`; `load_custom_templates` skips the file |
| `~/.config/chx/templates/` directory doesn't exist | `load_custom_templates` returns empty vec |
| Empty magic bytes in template | `matches` always returns false |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `serde` | `Deserialize` derive for TOML parsing helper structs |
| `toml` | Parsing custom TOML template files |
| `std::collections::HashMap` | `build_field_map` offset lookup |
| `std::fs` / `std::path` | Reading custom template files from disk |

## Change Log

| Date | Description |
|------|-------------|
| 2026-03-30 | Initial spec for format template system |
