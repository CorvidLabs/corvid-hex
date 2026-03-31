//! # chx — A TUI Hex Editor
//!
//! `chx` is a terminal-based hex editor built with [ratatui](https://ratatui.rs/).
//! It supports viewing and editing binary files with hex and ASCII panes,
//! search (hex and text), mouse navigation, and configurable column widths.

mod app;
mod buffer;
mod diff;
mod diff_render;
mod entropy;
mod format;
mod input;
mod inspector;
mod render;
mod search;
mod strings;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use diff::DiffState;
use ratatui::prelude::*;
use std::io;

#[derive(Parser)]
#[command(name = "chx", about = "A TUI hex editor")]
struct Cli {
    /// File to open (or first file in diff mode)
    file: String,

    /// Bytes per row (default: 16)
    #[arg(short = 'c', long = "columns", default_value_t = 16)]
    columns: usize,

    /// Compare two files (binary diff mode)
    #[arg(short = 'd', long = "diff")]
    diff_file: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(ref right_path) = cli.diff_file {
        // Diff mode
        let mut state = DiffState::open(&cli.file, right_path)?;
        state.bytes_per_row = cli.columns.max(1);

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = run_diff(&mut terminal, &mut state);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
        terminal.show_cursor()?;

        if result.is_ok() {
            let stats = state.stats();
            eprintln!(
                "Diff summary: {} bytes compared, {} differences ({:.1}% match)",
                stats.total_bytes, stats.diff_count, stats.match_percentage
            );
        }
        result
    } else {
        // Normal editor mode
        let mut app = App::open(&cli.file)?;
        app.bytes_per_row = cli.columns.max(1);
        app.requested_bytes_per_row = app.bytes_per_row;

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = run(&mut terminal, &mut app);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
        terminal.show_cursor()?;

        result
    }
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| render::draw(f, app))?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if input::handle_key(app, key) {
                    break;
                }
            }
            Event::Mouse(mouse) => {
                input::handle_mouse(app, mouse);
            }
            _ => {}
        }
    }
    Ok(())
}

fn run_diff(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut DiffState,
) -> Result<()> {
    // Show initial stats
    let stats = state.stats();
    state.status_message = Some(format!(
        "{} diffs, {:.1}% match — ]c/[c: next/prev diff, x: XOR view, q: quit",
        stats.diff_count, stats.match_percentage
    ));

    // Track pending bracket key for [c / ]c sequences
    let mut pending_bracket: Option<char> = None;

    loop {
        terminal.draw(|f| diff_render::draw_diff(f, state))?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Handle pending bracket sequences
                if let Some(bracket) = pending_bracket.take() {
                    match key.code {
                        KeyCode::Char('c') => {
                            if bracket == ']' {
                                state.next_diff();
                            } else {
                                state.prev_diff();
                            }
                        }
                        _ => {
                            // Invalid second key — ignore
                        }
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,

                    // Navigation
                    KeyCode::Char('h') | KeyCode::Left => state.move_cursor(-1),
                    KeyCode::Char('l') | KeyCode::Right => state.move_cursor(1),
                    KeyCode::Char('k') | KeyCode::Up => {
                        state.move_cursor(-(state.bytes_per_row as isize));
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        state.move_cursor(state.bytes_per_row as isize);
                    }

                    KeyCode::Char('g') => state.move_cursor_to(0),
                    KeyCode::Char('G') => {
                        if state.max_len() > 0 {
                            state.move_cursor_to(state.max_len() - 1);
                        }
                    }

                    KeyCode::PageDown => state.page_down(),
                    KeyCode::PageUp => state.page_up(),

                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let half = (state.visible_rows / 2) * state.bytes_per_row;
                        state.move_cursor(half as isize);
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let half = (state.visible_rows / 2) * state.bytes_per_row;
                        state.move_cursor(-(half as isize));
                    }

                    // Diff navigation: ]c and [c
                    KeyCode::Char(']') => {
                        pending_bracket = Some(']');
                    }
                    KeyCode::Char('[') => {
                        pending_bracket = Some('[');
                    }

                    // Shortcut: n/N for next/prev diff
                    KeyCode::Char('n') => state.next_diff(),
                    KeyCode::Char('N') => state.prev_diff(),

                    // Toggle XOR view
                    KeyCode::Char('x') => state.toggle_xor_view(),

                    _ => {}
                }
            }
            Event::Mouse(mouse) => {
                use crossterm::event::MouseEventKind;
                match mouse.kind {
                    MouseEventKind::ScrollDown => {
                        state.move_cursor((3 * state.bytes_per_row) as isize);
                    }
                    MouseEventKind::ScrollUp => {
                        state.move_cursor(-((3 * state.bytes_per_row) as isize));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}
