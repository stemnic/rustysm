use std::io;
use mpv;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::path;
use log::{error, info, warn, debug, trace};
use crate::external_program_status::*;

// Functionality
/*
    MPV playfile
    MPV seekfile
    MPV speed
    MPV audiopitchcorrection
    MPV Play/Pause

*/

struct MPVMessage {
    command: MPVCommand,
    message: String, // Can be nothing if command does not take any parameters
}

struct MPVFeedback {
    feedbacktype: MPVFeedbackType,
    message: String, // Can be nothing if type does not take any parameters
}

enum MPVCommand {
    Playfile,
    Seekfile,
    CycleSubtitles,
    Speed,
    DisableAudioPitchCorrection,
    EnableAudioPitchCorrection,
    Play,
    Pause,
}

enum MPVFeedbackType {
    Idle,
}



#[derive(Debug)]
pub struct Daemon{
    mpv_controller : Sender<MPVMessage>,
    mpv_feedback   : Receiver<MPVFeedback>,
}

impl Daemon{
    pub fn new() -> Result<Self, io::Error> {
        /*  TODO
            MPV backend object init
            socket connection for reciving commands
            Queue functionality
            Youtube download functionality
        */
        let (mpv_instance_tx, mpv_instance_rx) : (Sender<MPVMessage>, Receiver<MPVMessage>) = channel();
        let (mpv_feedback_tx, mpv_feedback_rx) : (Sender<MPVFeedback>, Receiver<MPVFeedback>) = channel();
        let mpv_thread = std::thread::spawn(move || {
            let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Failed to init MPV builder");
            // set option "sid" to "no" (no subtitles)
            mpv_builder.set_option("sid","no").unwrap();
            // enable On Screen Controller (disabled with libmpv by default)
            mpv_builder.set_option("osc",true).unwrap();
            let mut mpv = mpv_builder.build().expect("Failed to build MPV handler");
            mpv.set_option("idle", "yes").unwrap();
            let mut spotify_was_playing = false;
            let mut mpd_was_playing = false;

            'main: loop {
                while let Some(event) = mpv.wait_event(0.0) {
                    // even if you don't do anything with the events, it is still necessary to empty
                    // the event loop

                    log::trace!("RECEIVED EVENT : {:?}", event);
                    match event {
                        // Shutdown will be triggered when the window is explicitely closed,
                        // while Idle will be triggered when the queue will end
                        mpv::Event::Shutdown => {
                            println!("MPV shutting down!");
                            break 'main;
                        }
                        mpv::Event::Idle => {
                            println!("Spotify {}", spotify_was_playing);
                            println!("mpd {}", mpd_was_playing);
                            if spotify_was_playing {
                                spotify_was_playing = false;
                                spotify_play_pause();
                            }
                            if mpd_was_playing {
                                mpd_was_playing = false;
                                mpd_play();
                            }
                            let message = MPVFeedback { feedbacktype: MPVFeedbackType::Idle, message: "".to_string()  };
                            mpv_feedback_tx.send(message).unwrap();
                        }
                        mpv::Event::Unpause | mpv::Event::StartFile => {
                            if spotify_playing() {
                                spotify_was_playing = true;
                                spotify_play_pause();
                            }
                            if mpd_playing() {
                                mpd_was_playing = true;
                                mpd_pause();
                            }
                        }
                        _ => {
                            println!("{:?} Got event", event);
                        }
                    };
                    
                }
                match mpv_instance_rx.try_recv() {
                    Ok(recv) => {
                        match recv.command {
                            MPVCommand::Pause       => mpv.set_property("pause", true).unwrap(),
                            MPVCommand::Play        => mpv.set_property("pause", false).unwrap(),
                            MPVCommand::Playfile    => {
                                let video_path = path::PathBuf::from(recv.message);
                                mpv.command(&["loadfile", &video_path.into_os_string().into_string().unwrap() as &str]).expect("Error loading file");
                            }
                            MPVCommand::Speed       => {
                                let speed = recv.message.parse::<f64>().unwrap();
                                mpv.set_property("speed", speed).unwrap();
                            }
                            MPVCommand::DisableAudioPitchCorrection => mpv.set_property("audio-pitch-correction", "no").unwrap(),
                            MPVCommand::EnableAudioPitchCorrection => mpv.set_property("audio-pitch-correction", "yes").unwrap(),
                            _ => ()
                        }
                    }
                    Err(_) =>{}
                }
            }
        });
        let com = Daemon{ mpv_controller: mpv_instance_tx, mpv_feedback: mpv_feedback_rx };
        Ok(com)
    }

    pub fn mpv_play_file(&self, file:&str) -> () {
        self.mpv_controller.send(MPVMessage { command: MPVCommand::Playfile, message: file.to_string()}).unwrap();
    }

    pub fn mpv_disable_audio_pitch_correction(&self) -> () {
        self.mpv_controller.send(MPVMessage { command: MPVCommand::DisableAudioPitchCorrection, message: "".to_string()}).unwrap();
    }
    pub fn mpv_enable_audio_pitch_correction(&self) -> () {
        self.mpv_controller.send(MPVMessage { command: MPVCommand::EnableAudioPitchCorrection, message: "".to_string()}).unwrap();
    }
    pub fn mpv_set_speed(&self, speed:f64) -> () {
        self.mpv_controller.send(MPVMessage { command: MPVCommand::Speed, message: speed.to_string()}).unwrap();
    }
    pub fn mpv_play_add(&self, file:&str) -> () {
        self.mpv_controller.send(MPVMessage { command: MPVCommand::Playfile, message: file.to_string()}).unwrap();
    }

}
