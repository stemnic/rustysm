
use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders, Gauge};
use tui::style::{Color, Modifier, Style};
use tui::layout::{Layout, Constraint, Direction};


fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    loop{
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("Fantastic box")
                .borders(Borders::ALL);
            let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ].as_ref())
            .split(f.size());
            
            let chunksHorisontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ].as_ref())
            .split(chunks[0]);

            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, chunksHorisontal[0]);
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, chunksHorisontal[1]);
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, chunksHorisontal[2]);
            let gauge = Gauge::default()
                .block(Block::default().title("Gauge1").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(50);
            f.render_widget(gauge, chunks[1]);

            
            let block = Block::default().title("Block 2").borders(Borders::ALL);
            f.render_widget(block, chunks[2]);

        })?
    }
}