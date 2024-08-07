use std::io;

#[cfg(target_os = "linux")]
mod alsa_controller;
mod daemon;
mod daemon_downloader;
mod daemon_queue;
mod external_program_status;
mod history_watcher;
mod socket_com;
mod status_watcher;
mod tab_elements;
mod terminal_ui;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

use clap::{App, Arg};
use dirs::home_dir;

use log::info;

use crate::socket_com::{SocketCom, DEFAULT_PRIORITY};

fn init_log(log_file_name: &str) -> () {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%m-%d %H:%M:%S)}:{f}#{L}:[{h({l})}] - {m}\n",
        )))
        .build(log_file_name)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(Logger::builder().build("rustysm::status_watcher", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::terminal_ui", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::history_watcher", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::alsa_controller", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::tab_elements", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::socket_com", LevelFilter::Info))
        .logger(Logger::builder().build("rustysm::daemon", LevelFilter::Info))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config).unwrap();
}

fn main() -> Result<(), io::Error> {
    let args = App::new("Rustysm")
                .version("0.1.4")
                .author("Ole Sivert Aarhaug <ole.sivert@gmail.com>, Tore Mattias Apeland <turtlesmoker@gmail.com>")
                .about("Rustyshowmovie allows you to queue and play videoes and other media files")
                .arg(Arg::with_name("gui")
                        .short("g")
                        .long("gui")
                        .takes_value(false)
                        .help("Launches rustysm in gui mode"))
                .arg(Arg::with_name("play")
                        .long("play")
                        .takes_value(false)
                        .help("Resumes the video in showmovie backend"))
                .arg(Arg::with_name("pause")
                        .long("pause")
                        .takes_value(false)
                        .help("Pauses playing video in showmovie backend"))
                .arg(Arg::with_name("daemon")
                        .short("d")
                        .long("daemon")
                        .takes_value(false)
                        .help("Launches rustysm daemon mode to recive commands from client"))
                .arg(Arg::with_name("tickrate")
                        .short("t")
                        .long("tickrate")
                        .takes_value(true)
                        .help("Adjusts the tickrate in ms of gui if terminal has a slow refresh rate"))
                .arg(Arg::with_name("logfile")
                        .short("l")
                        .long("logfile")
                        .takes_value(true)
                        .help("Path to desired placement of client logfile"))
                .arg(Arg::with_name("history_file")
                        .short("hist")
                        .long("history_file")
                        .takes_value(true)
                        .help("Provides the path to the history file."))
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
                        .help("Media file to add to queue")
                        .multiple(true))
                .get_matches();
    let mut log_path = home_dir().unwrap();
    if args.is_present("logfile") {
        log_path = std::path::PathBuf::from(args.value_of("logfile").unwrap())
    } else {
        log_path.push(".sm_client.log");
    }
    init_log(&log_path.into_os_string().into_string().unwrap());
    log::info!(
        "-----------------------------------------\n\n\nStarting great program, just for you!!"
    );
    if args.is_present("gui") {
        let mut tickrate = 10;
        let mut home_path = home_dir().unwrap();
        home_path.push(".sm_history");
        let mut history_file_path = home_path.into_os_string().into_string().unwrap();
        if args.is_present("tickrate") {
            tickrate = args.value_of("tickrate").unwrap().parse::<u64>().unwrap();
        }
        if args.is_present("history_file") {
            history_file_path = args.value_of("history_file").unwrap().to_string();
        }
        info!("Opening with history path {}", history_file_path);
        let mut ui = terminal_ui::TerminalUi::new(history_file_path)?;
        ui.start_draw(tickrate).unwrap();
    } else if args.is_present("daemon") {
        log::info!("Attempting to start daemon");
        let daemon = daemon::Daemon::new().unwrap();
        daemon.mpv_play_file("https://www.youtube.com/watch?v=138ajKRMzIY");
        daemon.mpv_disable_audio_pitch_correction();
        let mut speed = 0.5;
        let mut forward = true;
        loop {
            std::thread::sleep_ms(100);
            if forward {
                speed = speed + 0.01;
            } else {
                speed = speed - 0.01;
            }
            if speed >= 2.0 {
                forward = false;
                daemon.mpv_enable_audio_pitch_correction();
            }
            if speed <= 0.5 {
                forward = true;
                daemon.mpv_disable_audio_pitch_correction();
                speed = 0.5;
            }
            println!("{:?} {:?}", speed, forward);

            daemon.mpv_set_speed(speed);
        }
    } else if args.is_present("play") {
        log::info!("Resuming sm backend");
        let mut socket_controller = SocketCom::new().unwrap();
        socket_controller.start_playback();
    } else if args.is_present("pause") {
        log::info!("Pausing sm backend");
        let mut socket_controller = SocketCom::new().unwrap();
        socket_controller.pause_playback();
    } else if args.is_present("QueueFile") {
        let tbq = args.value_of("QueueFile").unwrap();
        let mut priority = DEFAULT_PRIORITY;
        if args.is_present("priority") {
            priority = args.value_of("priority").unwrap().parse::<u64>().unwrap();
        }
        let mut socket_controller = SocketCom::new().unwrap();
        for object_to_be_queued in args.values_of("QueueFile").unwrap().collect::<Vec<_>>() {
            let result_msg = match socket_controller.add_entry(object_to_be_queued.to_string(), priority, args.is_present("raw")) {
                Ok(value) => value,
                Err(_) => {"Could not successfully queue object.\nConsider using -r to try as filestream for livestreams or other types.".to_string()}
            };
            println!("{}", result_msg);
        }
    } else {
        println!("No input provided, consider trying --help");
    }
    Ok(())
}
