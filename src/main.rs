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
mod alsa_controller;
mod tab_elements;
mod socket_com;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};

use dirs::home_dir;

fn init_log(log_file_name : &str) -> () {
    let logfile = FileAppender::builder()
    .encoder(Box::new(PatternEncoder::new("{d(%m-%d %H:%M:%S)}:{f}#{L}:[{h({l})}] - {m}\n")))
    .build(log_file_name).unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(Logger::builder().build("rustysm::status_watcher", LevelFilter::Debug))
        .logger(Logger::builder().build("rustysm::terminal_ui", LevelFilter::Debug))
        .logger(Logger::builder().build("rustysm::history_watcher", LevelFilter::Debug))
        .logger(Logger::builder().build("rustysm::alsa_controller", LevelFilter::Debug))
        .logger(Logger::builder().build("rustysm::tab_elements", LevelFilter::Debug))
        .logger(Logger::builder().build("rustysm::socket_com", LevelFilter::Trace))
        .build(Root::builder()
                .appender("logfile")
                
                .build(LevelFilter::Info)).unwrap();
    log4rs::init_config(config).unwrap();
}

fn main() -> Result<(), io::Error> {
    let mut log_path = home_dir().unwrap();
    log_path.push(".rusty_log.log");
    init_log(&log_path.into_os_string().into_string().unwrap());

    log::info!("-----------------------------------------\n\n\nStarting great program, just for you!!");

    let mut ui = terminal_ui::TerminalUi::new()?;
    ui.start_draw();
    Ok(())
} 