use std::io;

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

use clap::{Arg, App, SubCommand};
use dirs::home_dir;

use crate::socket_com::{SocketCom, DEFAULT_PRIORITY};

fn init_log(log_file_name : &str) -> () {
    let logfile = FileAppender::builder()
    .encoder(Box::new(PatternEncoder::new("{d(%m-%d %H:%M:%S)}:{f}#{L}:[{h({l})}] - {m}\n")))
    .build(log_file_name).unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(Logger::builder().build("rustysm::status_watcher", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::terminal_ui", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::history_watcher", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::alsa_controller", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::tab_elements", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::socket_com", LevelFilter::Info))
        .build(Root::builder()
                .appender("logfile")
                
                .build(LevelFilter::Info)).unwrap();
    log4rs::init_config(config).unwrap();
}

fn main() -> Result<(), io::Error> {
    let mut log_path = home_dir().unwrap();
    log_path.push(".rusty_log.log");
    init_log(&log_path.into_os_string().into_string().unwrap());
    let args = App::new("Rustysm")
                .version("0.1.0")
                .author("Ole Sivert Aarhaug <ole.sivert@gmail.com>, Tore Mattias Apeland <turtlesmoker@gmail.com>")
                .about("Rustyshowmovie allows you to queue and play videoes and other media files")
                .arg(Arg::with_name("gui")
                        .short("g")
                        .long("gui")
                        .takes_value(false)
                        .help("Launches rustysm in gui mode"))
                .arg(Arg::with_name("tickrate")
                        .short("t")
                        .long("tickrate")
                        .takes_value(true)
                        .help("Adjusts the tickrate in ms of gui if terminal has a slow refresh rate"))
                .arg(Arg::with_name("priority")
                        .short("p")
                        .long("priority")
                        .takes_value(true)
                        .help("Set priority of the queued file"))
                .arg(Arg::with_name("raw")
                        .short("r")
                        .long("raw")
                        .takes_value(false)
                        .help("Forward input as is without parsing. Useful to play streams and such through mpv"))
                .arg(Arg::with_name("QueueFile")
                        .required(false)
                        .index(1)
                        .help("Media file to add to queue"))
                .get_matches();
    log::info!("-----------------------------------------\n\n\nStarting great program, just for you!!");
    if args.is_present("gui") {
        let mut tickrate = 10;
        if args.is_present("tickrate") {
            tickrate = args.value_of("tickrate").unwrap().parse::<u64>().unwrap();
        }
        let mut ui = terminal_ui::TerminalUi::new()?;
        ui.start_draw(tickrate).unwrap();
    } else if args.is_present("QueueFile") {
        let tbq = args.value_of("QueueFile").unwrap();
        println!("Trying to add: {}", tbq);
        let mut priority = DEFAULT_PRIORITY;
        if args.is_present("priority") {
            priority = args.value_of("priority").unwrap().parse::<u64>().unwrap();
        }
        let mut socket_controller = SocketCom::new().unwrap();
        socket_controller.add_entry(tbq.to_string(), priority, args.is_present("raw")).unwrap();

    } else {
        println!("No input provided, consider trying --help");
    }
    Ok(())
} 