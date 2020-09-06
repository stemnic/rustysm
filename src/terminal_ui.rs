use std::io;
use termion::raw::IntoRawMode;
use tui::backend::Backend;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders, Gauge};
use tui::style::{Color, Modifier, Style};
use tui::layout::{Layout, Constraint, Direction};
use tui::Frame;
use std::path;
use std::sync::mpsc::*;
use std::thread;

use std::process;

use termion::event::Key;
use termion::input::TermRead;

use crate::status_watcher::StatusWatcher;


#[derive(Debug)]
struct TuiState {
    playback_position_percent: f64, // #TODO: Should be Duration when backend supports actual playback

}

pub struct TerminalUi
{
    terminal: tui::Terminal<tui::backend::TermionBackend<termion::raw::RawTerminal<std::io::Stdout>>>,
    current_status: StatusWatcher
}


impl TerminalUi 
{
    pub fn new() -> Result<Self, io::Error> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let terminal_backend = Terminal::new(backend)?;
        let mut tui_ui = TerminalUi { terminal: terminal_backend , current_status: StatusWatcher::new(path::PathBuf::from("/tmp/smqueue.status"), path::PathBuf::from("/tmp/smqueue.queue"))? };
        tui_ui.current_status.start();
        tui_ui.terminal.clear().unwrap();
        Ok(tui_ui)
    }

    pub fn start_draw(&mut self) {
        let (tx, rx) = channel();

        thread::spawn(move || {
            let stdin = io::stdin();
            for evt in stdin.keys() {
                if let Ok(key) = evt {
                    if let Err(err) = tx.send(key) {
                        eprintln!("{}", err);
                        return;
                    }
                }
            }
        });
        loop{
            /*
            let stdin = io::stdin();
            for evt in stdin.keys() {
                if let Ok(key) = evt {
                    //println!("Got {:?}", key);
                    if key == termion::event::Key::Ctrl('c'){
                        process::exit(0);
                    }
                    /*
                    if let Err(err) = tx.send(Event::Input(key)) {
                        eprintln!("{}", err);
                        return;
                    }
                    if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                        return;
                    }
                    */
                }
            }
            */
            if let Ok(event) = rx.try_recv(){
                match event {
                    termion::event::Key::Ctrl('c') | termion::event::Key::Char('q') => {
                        self.terminal.clear().unwrap();
                        break;
                    }
                    _ => {}
                }
            }
            
            self.terminal.draw(|f| {
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
    
            }).unwrap();
        }
    }

}

