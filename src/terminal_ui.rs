use std::io;
use termion::raw::IntoRawMode;
use tui::backend::Backend;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders, Gauge, LineGauge, Tabs, Table, Row, Cell, TableState, Wrap, Paragraph, BarChart};
use tui::style::{Color, Modifier, Style};
use tui::layout::{Alignment, Layout, Constraint, Direction};
use tui::Frame;
use tui::text::Spans;
use tui::text::Span;
use tui::symbols;
use std::path;
use std::sync::mpsc::*;
use std::thread;
use alsa::mixer;
use alsa::Round;
use std::f32;
use std::u32;
use std::time::Duration;

use std::process;

use termion::event::Key;
use termion::input::TermRead;

use crate::status_watcher::StatusWatcher;
use crate::status_watcher::HistoryLog;


#[derive(Debug)]
struct TuiState {
    playback_position_percent: f64, // #TODO: Should be Duration when backend supports actual playback

}

pub struct TerminalUi
{
    terminal: tui::Terminal<tui::backend::TermionBackend<termion::raw::RawTerminal<std::io::Stdout>>>,
    current_status: StatusWatcher,
    history_log: HistoryLog,
}

fn get_normalized_volume(min: f32, max: f32, value: f32) -> f32{
    // Based upon https://github.com/alsa-project/alsa-utils/blob/9b02b42db48d4c202afcaf5001019534f30f6c96/alsamixer/volume_mapping.c#L83-L118
    // Does not work 100%, though well enough
    let normalized = f32::powf(10.0,(value - max) / 6000.0);
    let min_norm = f32::powf(10.0, (min - max) / 6000.0);
    let normalized = (normalized - min_norm) / (1.0 - min_norm);
    //let pos = min.to_db() - value.to_db();
    //normalized = 1.0 - f32::powf(2.0, (pos)/(8.0));
    return normalized
}


impl TerminalUi 
{
    pub fn new() -> Result<Self, io::Error> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let terminal_backend = Terminal::new(backend)?;
        let mut tui_ui = TerminalUi { terminal: terminal_backend , 
                                    current_status: StatusWatcher::new(path::PathBuf::from("/tmp/smqueue.status"), path::PathBuf::from("/tmp/smqueue.queue"))?,
                                    history_log : HistoryLog::new(path::PathBuf::from("/tmp/smqueue.history"))? 
        };
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
        let mut gauge_pros = 50;
        let mut queue_list_pos = 0;
        let mut tab_select = 0;
        let mut volume_percentage = 50.0;
        let mut volume_db = 0.0;
        
        self.history_log.read(100,0);
        loop{
            let mixer = mixer::Mixer::new("default", true).unwrap();
            let mixer_select = mixer::SelemId::new("Master", 0);
            let mixer_channel = match mixer.find_selem(&mixer_select) {
                Some(value) => {value}
                None => {panic!("Failed to open alsa interface")}
            };
            let (mixer_db_min, mixer_db_max) = mixer_channel.get_playback_db_range();
            let (mixer_vol_min, mixer_vol_max) = mixer_channel.get_playback_volume_range();

            let volume_mb = mixer_channel.get_playback_vol_db(mixer::SelemChannelId::Last).unwrap();
            volume_db = mixer_channel.get_playback_vol_db(mixer::SelemChannelId::Last).unwrap().to_db();
            volume_percentage = 1.0 - (volume_db / mixer_db_min.to_db());
            let playback_percentage = self.current_status.status_info.lock().unwrap().playback_time;
            let queue_list = self.current_status.status_info.lock().unwrap().entry_list.clone();
            let playback_state = self.current_status.status_info.lock().unwrap().playback_state.clone();
            let mut queue_size = 0;
            if queue_list.len() > 0 {
                queue_size = queue_list.len()-1;
            }

            if let Ok(event) = rx.try_recv(){
                while let Ok(_) = rx.try_recv(){
                    // clear input buffer so there is no lag feel if buttons are held inn
                }
                match event {
                    termion::event::Key::Ctrl('c') | termion::event::Key::Char('q') | termion::event::Key::Esc => {
                        self.terminal.clear().unwrap();
                        break;
                    }
                    termion::event::Key::Char('1')  => tab_select = 0,
                    termion::event::Key::Char('2')  => tab_select = 1,
                    termion::event::Key::Char('3')  => tab_select = 2,
                    termion::event::Key::Char('4')  => tab_select = 3,
                    termion::event::Key::Char('h')  => tab_select = 3,
                    termion::event::Key::Char('?')  => tab_select = 3,
                    termion::event::Key::F(1)       => tab_select = 3,
                    termion::event::Key::Right => {
                        if gauge_pros < 100 {
                            gauge_pros = gauge_pros + 1;
                        }
                    }
                    termion::event::Key::Left => {
                        if gauge_pros > 0 {
                            gauge_pros = gauge_pros - 1;
                        }
                    }
                    termion::event::Key::Down => {
                        if queue_list_pos < queue_size {
                            queue_list_pos = queue_list_pos + 1;
                        }
                    }

                    termion::event::Key::Char('+') => {
                        if volume_percentage < 1.0 {
                            let to_db_part = ((1.0 - volume_percentage) - 0.01) * mixer_db_min.to_db();
                            let to_db = mixer::MilliBel::from_db(to_db_part);
                            mixer_channel.set_playback_db_all(to_db, Round::Floor).unwrap();
                        }
                    }

                    termion::event::Key::Char('-') => {
                        if volume_percentage > 0.0 {
                            let to_db_part = ((1.0 - volume_percentage) + 0.01) * mixer_db_min.to_db();
                            let to_db = mixer::MilliBel::from_db(to_db_part);
                            mixer_channel.set_playback_db_all(to_db, Round::Floor).unwrap();
                        }
                    }

                    termion::event::Key::PageDown => {
                        if queue_list_pos < queue_size &&  queue_list_pos < queue_size-10{
                            queue_list_pos = queue_list_pos + 10;
                            if queue_list_pos == queue_size {
                                queue_list_pos = queue_size;
                            }
                        } else {
                            queue_list_pos = queue_size;
                        }
                    }
                    termion::event::Key::Up => {
                        if queue_list_pos > 0 {
                            queue_list_pos = queue_list_pos - 1;
                        }
                    }
                    termion::event::Key::PageUp => {
                        if queue_list_pos > 0 && queue_list_pos > 10 {
                            queue_list_pos = queue_list_pos - 10;
                            if queue_list_pos == 0 {
                                queue_list_pos = 0;
                            }
                        } else {
                            queue_list_pos = 0;
                        }
                    }
                    termion::event::Key::Char('p') => {
                        println!("{:?}", self.current_status);
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
                    Constraint::Length(3),
                    Constraint::Length(2),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Percentage(80),
                ].as_ref())
                .split(f.size());

                let playback_gauge = LineGauge::default()
                    .block(Block::default().borders(Borders::BOTTOM).title(playback_state.to_string()))
                    .gauge_style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD))
                    .line_set(symbols::line::ROUNDED)
                    .ratio((playback_percentage as f64)/100.0);
                f.render_widget(playback_gauge, chunks[0]);

                let volume_gauge = LineGauge::default()
                    .block(Block::default().borders(Borders::NONE).title("Volume | ".to_string() + &volume_db.to_string() + "dB / " + &(mixer_db_min.to_db()).to_string() + "dB"))
                    .gauge_style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD))
                    .line_set(symbols::line::ROUNDED)
                    .ratio(get_normalized_volume(mixer_db_min.to_db(), mixer_db_max.to_db(), volume_mb.to_db()) as f64);
                f.render_widget(volume_gauge, chunks[1]);
                
                let paragraph = Paragraph::new("👉👉👉 h, 4, F1 or ? for help 👈👈👈".to_string())
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true });
                f.render_widget(paragraph, chunks[2]);

                /*
                let chunksHorisontal = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10),
                ].as_ref())
                .split(chunks[2]);
                */
                /*
                let block = Block::default().title("Block").borders(Borders::ALL);
                f.render_widget(block, chunksHorisontal[0]);
                */
                /*
                let block = Block::default().title("Block").borders(Borders::ALL);
                f.render_widget(block, chunksHorisontal[1]);
                let block = Block::default().title("Block").borders(Borders::ALL);
                f.render_widget(block, chunksHorisontal[2]);
                */
                /*
                let gauge = Gauge::default()
                    .block(Block::default().title("Volume").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Green).bg(Color::Gray))
                    .percent(gauge_pros);
                f.render_widget(gauge, chunks[0]);
                */
                let titles = ["Queue", "Hmm", "lalala", "Help"].iter().cloned().map(Spans::from).collect();
                let tabs = Tabs::new(titles)
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .select(tab_select)
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .divider(symbols::line::VERTICAL);
                f.render_widget(tabs, chunks[3]);

                /*
                let line_gauge = LineGauge::default()
                    .block(Block::default().borders(Borders::ALL).title("Progress"))
                    .gauge_style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD))
                    .line_set(symbols::line::ROUNDED)
                    .ratio((gauge_pros as f64)/100.0);
                f.render_widget(line_gauge, chunks[1]);
                */


                let mut rows = vec![];
                let mut first = true;

                for line in queue_list {
                    let mut style  = Style::default().fg(Color::Gray);
                    if first {
                        style  = Style::default().fg(Color::Yellow);
                        first = false;
                    }
                    rows.push(Row::new(vec![line.priority.to_string(), line.entry_type, line.file_location]).style(style))
                }
                
                let table = Table::new(rows)
                // You can set the style of the entire Table.
                .style(Style::default().fg(Color::White))
                // It has an optional header, which is simply a Row always visible at the top.
                .header(
                    Row::new(vec!["Pri", "Type", "Location"])
                        .style(Style::default().fg(Color::Yellow))
                        // If you want some space between the header and the rest of the rows, you can always
                        // specify some margin at the bottom.
                        .bottom_margin(1)
                )
                // As any other widget, a Table can be wrapped in a Block.
                .block(Block::default().borders(Borders::ALL).title("Queue 🤔🤔🤔🤔🤔"))
                // Columns widths are constrained in the same way as Layout...
                .widths(&[Constraint::Percentage(3), Constraint::Percentage(13), Constraint::Percentage(84)])
                // ...and they can be separated by a fixed spacing.
                .column_spacing(1)
                // If you wish to highlight a row in any specific way when it is selected...
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                // ...and potentially show a symbol in front of the selection.
                .highlight_symbol(">>");

                let mut state = TableState::default();
                state.select(Some(queue_list_pos));
                
                match tab_select {
                    0 => f.render_stateful_widget(table, chunks[4], &mut state),
                    1 => {},
                    _ => {}
                }
                

                //let block = Block::default().title("Block 2").borders(Borders::ALL);
                //f.render_widget(block, chunks[2]);
    
            }).unwrap();
            
            
        }
    }

}

