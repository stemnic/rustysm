use std::io;

#[derive(Debug)]
pub struct DaemonQueue {
    media_queue: Vec<QueueObject>,
    to_be_processed_queue: Vec<PreQueueObject>,
}

#[derive(Debug)]
struct QueueObject {
    id: u64,
    priority: u64,
    path: String, // Filepath, url, whater
    title: String,
    object_type: ObjectTypes,
    playback_postition: f64,
    settings: Settings,
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

#[derive(Debug)]
/// Input proccessed before adding to media queue
pub struct InputObject {
    input_string: String,
    priority: u64,
    settings: Settings,
}

#[derive(Debug)]
struct PreQueueObject {
    input: InputObject,
    finished: bool,
}

/*
1. Input object recived
2. Determines if it's a local file or downloadable file based on input
3. Add result to media queue
Alternaitve
1. Get file to play, start downloading it and wait with mpv playing until the download daemon have started.
*/

impl DaemonQueue {
    pub fn new() -> Result<Self, io::Error> {
        Ok(DaemonQueue {
            media_queue: vec![],
            to_be_processed_queue: vec![],
        })
    }
    pub fn add_to_queue(&mut self, object: InputObject) {
        // When you attempt to add a object to the queue it needs to be processed first
        self.to_be_processed_queue.push(PreQueueObject {
            input: object,
            finished: false,
        });
    }
    pub fn process_prequeue(&mut self) {}
}
