use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders, Gauge};
use tui::style::{Color, Modifier, Style};
use tui::layout::{Layout, Constraint, Direction};



use termion::event::Key;
use termion::input::TermRead;

use std::path;

mod status_watcher;
mod terminal_ui;
mod history_watcher;

fn main() -> Result<(), io::Error> {
    let mut ui = terminal_ui::TerminalUi::new()?;
    ui.start_draw();
    Ok(())
} 