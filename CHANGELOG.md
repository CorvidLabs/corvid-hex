# Changelog

All notable changes to chx will be documented in this file.

## [0.2.0] - 2026-03-30

### Added

- **Binary diff mode** — Compare two files side-by-side with `chx file1 -d file2`. Includes byte-by-byte comparison, diff navigation (`]c`/`[c`, `n`/`N`), and XOR view toggle (`x`).
- **Data inspector panel** — Multi-type byte interpretation at cursor position. Displays u8–u64, i8–i64, f32/f64 in both endiannesses, plus ASCII, binary, and octal representations. Toggle with `:inspector`, navigate with `j`/`k`, edit values with `Enter`.
- **Entropy visualization** — Shannon entropy heatmap overlay showing data randomness per block. Toggle with `:entropy`.
- **String extraction** — `:strings` command to scan and browse ASCII, UTF-8, and UTF-16LE/BE strings with configurable minimum length.
- **Format template system** — Auto-detect and label known binary headers (PNG, ZIP, ELF, PE, Mach-O, SQLite, JPEG, GIF, BMP, WAV, PDF). Custom templates via TOML files in `~/.config/chx/templates/`.
- **Memory-mapped I/O** — Large files (>100 MB) loaded via `mmap` for near-instant open times. Smaller files use the original copy-on-open model.
- **Mouse support** — Click to position cursor, drag to select (visual mode), scroll wheel navigation.

### Fixed

- Terminal width panic when window is narrower than hex view layout.

### Changed

- Removed bare `q` quit from normal mode — now requires `:q` + Enter for safety.

## [0.1.0] - 2026-03-29

### Added

- Initial release.
- Vi-style modal hex editor with Normal, Edit (Hex/ASCII), Visual, Command, and Search modes.
- Dual-pane hex + ASCII view with syntax coloring.
- ASCII and hex pattern search with case-insensitive (`/i`) and incremental highlighting.
- Search and replace (`:s/find/replace`).
- Undo/redo edit history.
- Visual select, yank, and paste.
- Offset bookmarks (`m<a-z>`, `'<a-z>`, `:marks`).
- Configurable bytes per row (`-c` flag, `:columns` command).
- Cross-platform release builds (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64).
- CI pipeline with tests, clippy, and spec-sync validation.
- GitHub Pages with Rust API documentation.
