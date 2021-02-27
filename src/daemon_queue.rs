use std::io;

use log::{error, info, warn, debug, trace};

use crate::daemon_downloader::*;

#[derive(Debug)]
pub struct DaemonQueue{
    queue: Vec<QueueObject>
}

#[derive(Debug)]
struct QueueObject {
    id: u64,
    priority: u64,
    source: String, // Filepath, url, whater
    object_type: ObjectTypes,
    playback_postition: f64,
    settings: Settings
}

#[derive(Debug)]
enum ObjectTypes {
    LocalFile,
    YoutubeVideo,
    FileStream,
}

#[derive(Debug)]
struct Settings {
    playback_speed: f64,
    audio_pitch_correction: bool,
    start_pos: f64,
    duration: f64,
    // Raw set properties for mpv vec with a str and template element?
}

/// Input proccessed before adding to media queue
struct InputObject {
    input_string: String,
    priority: u64,
    settings: Settings
}

/* 
1. Input object recived
2. Determines if it's a local file or downloadable file based on input
3. Add result to media queue
*/

impl DaemonQueue {
    pub fn new() -> Result<Self, io::Error> {
        Ok(DaemonQueue{ queue: vec![] })
    }
}