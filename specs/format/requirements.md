---
spec: format.spec.md
---

## User Stories

- As a user, I want common binary formats auto-detected so that I can see labeled fields without manual setup
- As a user, I want to define custom TOML templates so that I can label proprietary or uncommon formats
- As a user, I want field values decoded with proper endianness so that I can read integer and string values in context

## Acceptance Criteria

- Built-in templates cover PNG, ZIP, ELF, PE, Mach-O, SQLite, JPEG, GIF, BMP, WAV, PDF
- Format detection tries user templates before built-ins, allowing overrides
- PNG and ZIP templates dynamically resolve chunk/entry fields beyond static headers
- Custom templates are loaded from `~/.config/chx/templates/` as TOML files
- Invalid custom templates are silently skipped without crashing
- Field types support u8, u16/u32/u64 in both endianness, ASCII strings, and raw bytes
