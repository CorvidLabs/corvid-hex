---
spec: format.spec.md
---

## Key Decisions

- Magic-byte detection with optional secondary magic: some formats (e.g. WAV with RIFF+WAVE headers) need two checks at different offsets.
- Dynamic chunk resolution for container formats: PNG and ZIP have variable-length records that are walked at runtime rather than statically defined.
- First-wins semantics in `build_field_map`: when fields overlap, the first field in template order claims contested byte offsets.
- TOML custom templates use a `serde::Deserialize` intermediate struct (`TomlTemplate`/`TomlField`) that maps to the internal types.
- `load_custom_templates` uses silent error handling — unparseable files are skipped so one bad template doesn't break the entire system.

## Files to Read First

- `src/format.rs` — complete module: field types, template struct, built-in definitions, magic detection, TOML parsing, PNG/ZIP chunk walking
