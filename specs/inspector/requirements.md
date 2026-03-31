---
spec: inspector.spec.md
---

## User Stories

- As a user, I want to see the bytes at my cursor interpreted as various data types so that I can understand the data without manual conversion
- As a user, I want both little-endian and big-endian interpretations so that I can work with files from different architectures

## Acceptance Criteria

- Displays u8, i8, binary, octal, ASCII, and UTF-8 interpretations of the byte at cursor
- Displays u16/i16/u32/i32/f32/u64/i64/f64 in both little-endian and big-endian
- Gracefully handles insufficient bytes (e.g., cursor near end of file)
- Inspector panel updates live as cursor moves
- Editable fields allow modifying values and writing back to the buffer

## Constraints

- Must handle all standard IEEE 754 float edge cases (NaN, Inf, denormals)

## Out of Scope

- Custom/user-defined type interpretations
- Struct-level parsing (that's the format template system)
