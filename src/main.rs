//! # chx — A TUI Hex Editor
//!
//! `chx` is a terminal-based hex editor built with [ratatui](https://ratatui.rs/).
//! It supports viewing and editing binary files with hex and ASCII panes,
//! search (hex and text), mouse navigation, and configurable column widths.

mod app;
mod buffer;
mod format;
mod input;
mod render;
mod search;
mod strings;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

#[derive(Parser)]
#[command(name = "chx", about = "A TUI hex editor")]
struct Cli {
    /// File to open
    file: String,

    /// Bytes per row (default: 16)
    #[arg(short = 'c', long = "columns", default_value_t = 16)]
    columns: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
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
