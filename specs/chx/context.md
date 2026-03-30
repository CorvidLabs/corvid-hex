# chx — Context

## Key Decisions
- Copy-on-open model: entire file read into memory, edits stored in HashMap overlay
- Vim-style modal editing with 5 modes
- 16 bytes per row, fixed (not configurable yet)
- Overwrite-only editing (no insert/delete)

## Files to Read First
- `src/app.rs` — Application state and command execution
- `src/main.rs` — Entry point and event loop

## Current Status
MVP complete — all core features implemented and building clean.

## Notes
- Binary name is `chx` for fast typing
- Release profile uses LTO and strip for small binary size
