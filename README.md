# chx

A fast, modern TUI hex editor with Vi-style keybindings. Built in Rust with [ratatui](https://github.com/ratatui/ratatui).

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![CI](https://github.com/CorvidLabs/corvid-hex/actions/workflows/ci.yml/badge.svg)

## Features

### Core Editing
- **Vi-style modal editing** — Normal, Edit (Hex/ASCII), Visual, Command, and Search modes
- **Dual-pane view** — Hex and ASCII side-by-side with syntax coloring
- **Undo/redo** — Full edit history with `u` and `Ctrl-R`
- **Visual selection** — Select, yank (`y`), and paste (`p`) byte ranges
- **Bookmarks** — Set with `m<a-z>`, jump with `'<a-z>`, list with `:marks`
- **Configurable columns** — Set bytes per row via `-c` flag or `:columns` command
- **Mouse support** — Click to position, drag to select, scroll wheel navigation

### Search
- **ASCII and hex pattern search** — Incremental highlighting with case-insensitive (`/i`) support
- **Search and replace** — `:s/find/replace` for batch byte replacements

### Analysis Panels
- **Data inspector** — Multi-type byte interpretation at cursor (u8–u64, i8–i64, f32/f64, both endiannesses, ASCII, binary, octal)
- **Entropy visualization** — Shannon entropy heatmap overlay showing data randomness per block
- **String extraction** — `:strings` command to find and navigate ASCII, UTF-8, and UTF-16 strings
- **Format templates** — Auto-detect and label known binary headers (PNG, ZIP, ELF, PE, Mach-O, SQLite, JPEG, GIF, BMP, WAV, PDF) with custom TOML template support

### Binary Diff
- **Side-by-side diff mode** — Compare two files byte-by-byte with `chx file1 -d file2`
- **Diff navigation** — Jump between differences with `]c`/`[c` or `n`/`N`
- **XOR view** — Toggle XOR overlay to visualize byte differences

### Performance
- **Memory-mapped I/O** — Large files (>100 MB) loaded via `mmap` for near-instant open times
- **Efficient memory model** — Copy-on-open with sparse edit overlay for small files

## Installation

### From releases

Download a pre-built binary from [Releases](https://github.com/CorvidLabs/corvid-hex/releases) for:
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows (x86_64)

### From source

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
# Binary at target/release/chx
```

## Usage

```bash
chx <FILE> [-c <COLUMNS>]
chx <FILE> -d <FILE2>           # Binary diff mode
```

**Examples:**

```bash
chx firmware.bin                # Open with default 16 bytes per row
chx dump.bin -c 32              # Open with 32 bytes per row
chx old.bin -d new.bin          # Compare two files side-by-side
```

## Keybindings

### Normal Mode

| Key | Action |
|-----|--------|
| `h` `j` `k` `l` / Arrows | Navigate |
| `g` / `G` | Jump to start / end of file |
| `0` / `$` | Start / end of row |
| `Ctrl-D` / `Ctrl-U` | Half-page down / up |
| `PageDown` / `PageUp` | Full page down / up |
| `i` | Edit hex mode |
| `a` | Edit ASCII mode |
| `v` | Visual select mode |
| `/` | Search |
| `:` | Command mode |
| `n` / `N` | Next / previous search result |
| `u` / `Ctrl-R` | Undo / redo |
| `p` | Paste |
| `m<a-z>` | Set bookmark |
| `'<a-z>` | Jump to bookmark |
| `Mouse click` | Position cursor |
| `Mouse drag` | Select (visual mode) |
| `Scroll wheel` | Scroll up/down |

### Edit Modes

| Key | Action |
|-----|--------|
| Hex digits (Edit Hex) | Type two hex digits to write a byte |
| Any char (Edit ASCII) | Write ASCII byte at cursor |
| `Tab` | Toggle between Hex and ASCII editing |
| `Esc` | Return to Normal mode |

### Visual Mode

| Key | Action |
|-----|--------|
| Navigation keys | Extend selection |
| `y` | Yank selection |
| `Esc` | Cancel selection |

### Diff Mode

| Key | Action |
|-----|--------|
| `h` `j` `k` `l` / Arrows | Navigate |
| `]c` / `[c` | Next / previous difference |
| `n` / `N` | Next / previous difference (shortcut) |
| `x` | Toggle XOR view |
| `q` / `Esc` | Quit |

### Commands

| Command | Action |
|---------|--------|
| `:w` | Save |
| `:q` | Quit (fails if unsaved changes) |
| `:q!` | Force quit |
| `:wq` | Save and quit |
| `:goto <hex>` / `:g <hex>` | Jump to offset |
| `:s/find/replace` | Search and replace |
| `:columns <n>` / `:cols <n>` | Set bytes per row |
| `:marks` | List bookmarks |
| `:strings [min_len]` | Extract and browse strings |
| `:inspector` | Toggle data inspector panel |
| `:entropy` | Toggle entropy heatmap overlay |

### Search

| Pattern | Example | Description |
|---------|---------|-------------|
| ASCII | `/hello` | Search for text |
| ASCII (case-insensitive) | `/hello/i` | Case-insensitive text search |
| Hex | `/x/DEADBEEF` | Search for hex bytes |
| Hex (alt) | `/0xDEAD BEEF` | Hex with spaces or `0x` prefix |

## Custom Format Templates

Place TOML files in `~/.config/chx/templates/` to define custom binary format parsers:

```toml
name = "My Format"
magic = [0xDE, 0xAD, 0xBE, 0xEF]
magic_offset = 0

[[fields]]
name = "Header Magic"
offset = 0
size = 4
field_type = "bytes"

[[fields]]
name = "Version"
offset = 4
size = 2
field_type = "u16le"
```

Supported field types: `u8`, `u16le`, `u16be`, `u32le`, `u32be`, `u64le`, `u64be`, `ascii`, `bytes`.

## Color Scheme

| Color | Meaning |
|-------|---------|
| Cyan | Printable ASCII bytes |
| Dark gray | NULL bytes (0x00) |
| Yellow | Non-printable bytes / search matches |
| Red | Modified bytes / diff differences |
| Magenta | Active search match |
| Blue | Visual selection |
| Green | Cursor (edit modes) |

## Development

```bash
cargo build                # Build
cargo test                 # Run tests
cargo clippy               # Lint
```

The project uses [spec-sync](https://github.com/CorvidLabs/corvid-hex/tree/main/specs) for module documentation. Each module has a corresponding spec in `specs/`.

## Architecture

```
src/
  main.rs         Entry point, CLI args, terminal setup, diff loop
  app.rs          Application state machine and mode management
  buffer.rs       File I/O with mmap + sparse edit overlay
  input.rs        Keyboard/mouse event handling per mode
  render.rs       TUI rendering (header, hex view, panels, status bar)
  search.rs       Pattern matching, search/replace engine
  diff.rs         Binary diff engine (byte-by-byte comparison, XOR)
  diff_render.rs  Side-by-side diff TUI rendering
  entropy.rs      Shannon entropy calculation and heatmap
  inspector.rs    Multi-type data interpretation at cursor
  format.rs       Binary format template detection and parsing
  strings.rs      String extraction (ASCII, UTF-8, UTF-16)
```

## License

MIT
