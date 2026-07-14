## ADDED
### REQUIREMENT REQ-format-001
Built-in templates SHALL cover PNG, ZIP, ELF, PE, Mach-O, SQLite, JPEG, GIF, BMP, WAV, and PDF.

Acceptance Criteria
- Each listed format has a built-in detection template.

### REQUIREMENT REQ-format-002
Format detection SHALL try user templates before built-in templates so users can override detection.

Acceptance Criteria
- A matching user template is selected ahead of a matching built-in template.

### REQUIREMENT REQ-format-003
The PNG and ZIP templates SHALL dynamically resolve chunk or entry fields beyond static headers.

Acceptance Criteria
- PNG chunks and ZIP entries beyond their initial headers produce resolved fields.

### REQUIREMENT REQ-format-004
The application SHALL load custom TOML templates from `~/.config/chx/templates/`.

Acceptance Criteria
- A valid TOML template in that directory becomes available to detection.

### REQUIREMENT REQ-format-005
The application SHALL skip invalid custom templates without crashing.

Acceptance Criteria
- Malformed custom TOML does not terminate discovery or the application.

### REQUIREMENT REQ-format-006
Template fields SHALL support `u8`, `u16`, `u32`, and `u64` in both endiannesses, ASCII strings, and raw bytes.

Acceptance Criteria
- Every listed field kind decodes a representative valid byte sequence.

