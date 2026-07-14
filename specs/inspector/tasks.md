---
spec: inspector.spec.md
---

## Tasks

- [x] FieldType enum with all numeric and display types
- [x] byte_count method for each field type
- [x] is_editable classification (Binary/Octal/Ascii/Utf8 read-only)
- [x] parse method with hex prefix support for unsigned types
- [x] interpret function producing InspectorField list from raw bytes
- [x] UTF-8 multi-byte character detection
- [x] Comprehensive unit tests

## Gaps

- None identified in the current canonical behavior and native test suite.

## Review Sign-offs

- **Product evidence**: this migration preserves the documented product behavior.
- **QA evidence**: the repository-native Rust and Bun suites verify the existing behavior.
- **Design evidence**: no visual or interaction design changes are introduced.
- **Development evidence**: strict SpecSync, Clippy, and Trust gates verify the implementation boundary.
