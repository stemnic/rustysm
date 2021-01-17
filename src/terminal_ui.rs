use std::io;
use termion::raw::IntoRawMode;
use tui::backend::Backend;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders, Gauge, LineGauge, Tabs, Table, Row, Cell, TableState, Wrap, Paragraph, BarChart, Clear};
use tui::style::{Color, Modifier, Style};
use tui::layout::{Alignment, Layout, Constraint, Direction};
use tui::Frame;
use tui::text::{Spans, Span};
use tui::symbols;
use std::{path, thread};
use std::sync::mpsc::*;
use std::time::{Duration, Instant};

use termion::event::Key;
use termion::input::TermRead;

use crate::alsa_controller::AlsaController;
use crate::status_watcher::{StatusWatcher, PlaybackState};
use crate::history_watcher::{HistoryWatcher, DEFAULT_HISTORY_ENTRIES_TO_FETCH};
use crate::tab_elements::TabsElements;
use crate::socket_com::{SocketCom, EntryType, DEFAULT_PRIORITY};

use log::{error, info, warn, debug};

#[derive(Debug)]
struct TuiState {
    playback_position_percent: f64, // #TODO: Should be Duration when backend supports actual playback

}


pub struct TerminalUi
{
    terminal: tui::Terminal<tui::backend::TermionBackend<termion::raw::RawTerminal<std::io::Stdout>>>,
    current_status: StatusWatcher,
    history_log: HistoryWatcher,
}

impl TerminalUi 
{
    pub fn new() -> Result<Self, io::Error> {
        let stdout = io::stdout().into_raw_mode()?;
        let backend = TermionBackend::new(stdout);
        let terminal_backend = Terminal::new(backend)?;
        let mut tui_ui = TerminalUi { terminal: terminal_backend , 
                                    current_status: StatusWatcher::new(path::PathBuf::from("/tmp/smqueue.status"), path::PathBuf::from("/tmp/smqueue.queue"))?,
                                    history_log : HistoryWatcher::new(path::PathBuf::from("/tmp/smqueue.history"))? 
        };
        tui_ui.current_status.start();
        tui_ui.terminal.clear().unwrap();
        Ok(tui_ui)
    }

    pub fn start_draw(&mut self, tick_rate : u64) {
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
        let mut alsa_controller = AlsaController::new().unwrap();
        let mut socket_controller = SocketCom::new().unwrap();
        let mut queue_tab_element = TabsElements::new("Queue 🔜").unwrap();
        let mut history_tab_element = TabsElements::new("History 📜").unwrap();

        let tick_rate = Duration::from_millis(tick_rate);
        let mut last_tick = Instant::now();

        loop{

            
            let playback_percentage = self.current_status.status_info.lock().unwrap().playback_time;
            let queue_list = self.current_status.status_info.lock().unwrap().entry_list.clone();
            let playback_state = self.current_status.status_info.lock().unwrap().playback_state.clone();
            let mut queue_size = 0;
            if queue_list.len() > 0 {
                queue_size = queue_list.len()-1;
            }
            queue_tab_element.update_size(queue_size);
            self.history_log.read(DEFAULT_HISTORY_ENTRIES_TO_FETCH,0).unwrap(); // TODO: Have history make a notification when you need to update it
            let history_entries = self.history_log.entries.clone();
            history_tab_element.update_size(history_entries.len()-1);

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
                        if tab_select < 3 {
                            tab_select = tab_select + 1;
                        }
                    }
                    termion::event::Key::Left => {
                        if tab_select > 0 {
                            tab_select = tab_select - 1;
                        }
                    }
                    termion::event::Key::Char('+') => {
                        alsa_controller.volume_increment_db(1).unwrap();
                    }

                    termion::event::Key::Char('-') => {
                        alsa_controller.volume_decrement_db(1).unwrap();
                    }

                    termion::event::Key::Down => {
                        match tab_select {
                            0 => queue_tab_element.pos_down(),
                            1 => history_tab_element.pos_down(),
                            _ => {},
                        }
                    }

                    termion::event::Key::PageDown => {
                        match tab_select {
                            0 => queue_tab_element.pos_jump_down(10),
                            1 => history_tab_element.pos_jump_down(10),
                            _ => {},
                        }
                    }
                    termion::event::Key::Up => {
                        match tab_select {
                            0 => queue_tab_element.pos_up(),
                            1 => history_tab_element.pos_up(),
                            _ => {},
                        }
                    }
                    termion::event::Key::PageUp => {
                        match tab_select {
                            0 => queue_tab_element.pos_jump_up(10),
                            1 => history_tab_element.pos_jump_up(10),
                            _ => {},
                        }
                    }
                    termion::event::Key::Char(' ') => {
                        // Space
                        match playback_state {
                            PlaybackState::Playing => socket_controller.pause_playback().unwrap(),
                            _ => socket_controller.start_playback().unwrap()
                        }
                    }
                    termion::event::Key::Char('\t') => {
                        // Tab
                        socket_controller.skip_playback().unwrap();
                    }
                    termion::event::Key::Char('\n') => {
                        match tab_select {
                            0 => {
                                if queue_tab_element.table_list_size != 0 {
                                    let pos = queue_tab_element.table_list_pos;
                                    let queue_elem = queue_list[pos].clone();
                                    socket_controller.promote_entry(queue_elem.id).unwrap();
                                }
                            },
                            1 => {
                                if history_tab_element.table_list_size != 0 {
                                    let pos = history_tab_element.table_list_pos;
                                    let history_element = history_entries[pos].clone();
                                    socket_controller.add_entry(EntryType::LocalMedia, history_element.location, DEFAULT_PRIORITY).unwrap();
                                }
                            },
                            _ => {},
                        }
                    }
                    _ => {}
                }
            }
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
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
                        .block(Block::default().borders(Borders::NONE).title("Volume 🔊 ".to_string() + &alsa_controller.get_description_str() ))
                        .gauge_style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD))
                        .line_set(symbols::line::ROUNDED)
                        .ratio(alsa_controller.get_human_ear_volume_normalized());
                    f.render_widget(volume_gauge, chunks[1]);
                    
                    let paragraph = Paragraph::new("👉👉👉🆘 h, 4, F1 or ? for help 🆘👈👈👈".to_string())
                        .style(Style::default().fg(Color::Yellow))
                        .alignment(Alignment::Left)
                        .wrap(Wrap { trim: true });
                    f.render_widget(paragraph, chunks[2]);

                    
                    let titles = [&queue_tab_element.display_name, &history_tab_element.display_name, "lalala", "Help ❓"].iter().cloned().map(Spans::from).collect();
                    let tabs = Tabs::new(titles)
                        .block(Block::default().borders(Borders::ALL))
                        .style(Style::default().fg(Color::White))
                        .select(tab_select)
                        .highlight_style(Style::default().fg(Color::Yellow))
                        .divider(symbols::line::VERTICAL);
                    f.render_widget(tabs, chunks[3]);


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
                    .highlight_symbol("👉");

                    let mut rows_history = vec![];

                    for entry in history_entries {
                        let mut style  = Style::default().fg(Color::Gray);
                        rows_history.push(Row::new(vec![entry.timestamp, entry.name, entry.location]).style(style))
                    }
                    
                    let table_history = Table::new(rows_history)
                    .style(Style::default().fg(Color::White))
                    .header(
                        Row::new(vec!["Timestamp", "Name", "Location"])
                            .style(Style::default().fg(Color::Yellow))
                            .bottom_margin(1)
                    )
                    .block(Block::default().borders(Borders::ALL).title("History 📜"))
                    .widths(&[Constraint::Percentage(10), Constraint::Percentage(45), Constraint::Percentage(45)])
                    .column_spacing(1).highlight_style(Style::default().add_modifier(Modifier::BOLD)).highlight_symbol("👉");
                    let help_text = vec![
                        Spans::from(Span::styled("←/→ or Use the number row to go between tabs", Style::default().fg(Color::Gray))),
                        Spans::from(Span::styled("+/-: Adjusts volume on the system", Style::default().fg(Color::Gray))),
                        Spans::from(Span::styled("Esc/q/Ctrl-c: Quits this rusty application", Style::default().fg(Color::Gray))),
                        Spans::from(Span::styled("↑/↓: Move up and down in lists", Style::default().fg(Color::Gray))),
                        Spans::from(Span::styled("Pageup/Pagedown: Jump up and down in lists", Style::default().fg(Color::Gray))),
                        Spans::from(Span::styled("Space: Play/Pause playing media", Style::default().fg(Color::Gray))),
                        Spans::from(Span::styled("Enter: (Queue)Jump to or add to entry (History)", Style::default().fg(Color::Gray))),
                    ];
                    let help_block = Paragraph::new(help_text)
                        .block(Block::default().title("Help me").borders(Borders::ALL))
                        .style(Style::default().fg(Color::White).bg(Color::Black))
                        .alignment(Alignment::Left)
                        .wrap(Wrap { trim: true });
                    match tab_select {
                        0 => {
                            let mut state = TableState::default();
                            state.select(Some(queue_tab_element.table_list_pos));
                            f.render_stateful_widget(table, chunks[4], &mut state)
                        },
                        1 => {
                            let mut state = TableState::default();
                            state.select(Some(history_tab_element.table_list_pos));
                            f.render_stateful_widget(table_history, chunks[4], &mut state)
                        },
                        3 => f.render_widget(help_block, chunks[4]),
                        _ => {},
                    }
                }).unwrap();
            }
        }
    }

}

