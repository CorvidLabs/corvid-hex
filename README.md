# chx

A fast, modern TUI hex editor with Vi-style keybindings. Built in Rust with [ratatui](https://github.com/ratatui/ratatui).

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![CI](https://github.com/CorvidLabs/corvid-hex/actions/workflows/ci.yml/badge.svg)

## Features

- **Vi-style modal editing** — Normal, Edit (Hex/ASCII), Visual, Command, and Search modes
- **Dual-pane view** — Hex and ASCII side-by-side with syntax coloring
- **Search** — ASCII and hex pattern search with case-insensitive (`/i`) and incremental highlighting
- **Search and replace** — `:s/find/replace` for batch byte replacements
- **Undo/redo** — Full edit history with `u` and `Ctrl-R`
- **Visual selection** — Select, yank (`y`), and paste (`p`) byte ranges
- **Bookmarks** — Set with `m<a-z>`, jump with `'<a-z>`, list with `:marks`
- **Configurable columns** — Set bytes per row via `-c` flag or `:columns` command
- **Efficient memory model** — Copy-on-open with sparse edit overlay

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
```

**Examples:**

```bash
chx firmware.bin           # Open with default 16 bytes per row
chx dump.bin -c 32         # Open with 32 bytes per row
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

### Search

| Pattern | Example | Description |
|---------|---------|-------------|
| ASCII | `/hello` | Search for text |
| ASCII (case-insensitive) | `/hello/i` | Case-insensitive text search |
| Hex | `/x/DEADBEEF` | Search for hex bytes |
| Hex (alt) | `/0xDEAD BEEF` | Hex with spaces or `0x` prefix |

## Color Scheme

| Color | Meaning |
|-------|---------|
| Cyan | Printable ASCII bytes |
| Dark gray | NULL bytes (0x00) |
| Yellow | Non-printable bytes / search matches |
| Red | Modified bytes |
| Magenta | Active search match |
| Blue | Visual selection |
| Green | Cursor (edit modes) |

## Development

```bash
cargo build                # Build
cargo test                 # Run tests
cargo clippy               # Lint
```

The project uses [spec-sync](https://github.com/CorvidLabs/corvid-hex/tree/main/specs) for module documentation. Each module (`app`, `buffer`, `input`, `render`, `search`) has a corresponding spec in `specs/`.

## Architecture

```
src/
  main.rs     Entry point, CLI args, terminal setup
  app.rs      Application state machine and mode management
  buffer.rs   File I/O with copy-on-open + sparse edit overlay
  input.rs    Keyboard event handling per mode
  render.rs   TUI rendering (header, hex view, status bar)
  search.rs   Pattern matching, search/replace engine
```

## License

MIT
