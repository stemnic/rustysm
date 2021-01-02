// General
use std::string::String;
use std::io::Result;

// Threading
use std::sync::{Arc, Mutex};
use std::thread;

// File IO
use std::fs::File;
use std::io::prelude::*;
use std::path;

// File notification
use notify::{Watcher, RecursiveMode, raw_watcher, RawEvent};
use std::sync::mpsc::channel;
use std::fmt;
use std::io::{Error, ErrorKind};

#[derive(Debug, Clone)]
pub struct QueueEntry{
    pub id: u64,
    pub priority: u64,
    pub entry_type: String,
    pub file_location: String, // Can be local filepath or url
}

impl QueueEntry{
    pub fn new(id: u64, priority: u64, entry_type: String, file_location: String) -> Self {
        let stru = QueueEntry{
            id: id,
            priority: priority,
            entry_type: entry_type,
            file_location: file_location,
        };
        return stru;
    }
}

#[derive(Debug)]
pub struct QueueInfo {
    pub playback_state: PlaybackState,
    pub playback_time: f64,
    pub entry_list: Vec<QueueEntry>,
}

#[derive(Debug, Clone)]
pub enum PlaybackState {
    Playing,
    Paused,
    Idle, // Should be removed in backend implementation
    Stopped,
}

impl fmt::Display for PlaybackState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

impl QueueInfo {
    pub fn new() -> Self {
        let stru = QueueInfo{
            playback_state: PlaybackState::Stopped,
            playback_time: 0.0,
            entry_list: Vec::new(),
        };
        return stru;
    }
}

#[derive(Debug)]
pub struct StatusWatcher {
    sm_status_file: path::PathBuf,
    sm_queue_file: path::PathBuf,
    pub status_info: Arc<Mutex<QueueInfo>>,
}


impl StatusWatcher {
    pub fn new(status_file: path::PathBuf, queue_file: path::PathBuf) -> Result<Self>{
        let stru = StatusWatcher { 
            sm_status_file: path::PathBuf::from(status_file),
            sm_queue_file: path::PathBuf::from(queue_file),
            status_info:  Arc::new(Mutex::new(QueueInfo::new())),
        };
        Ok(stru)
    }

    pub fn start(&mut self) {
        let sm_status_file_copy = path::PathBuf::from(&self.sm_status_file);
        let sm_queue_file_copy = path::PathBuf::from(&self.sm_queue_file);

        let guarded_queue_info = self.status_info.clone();
        thread::spawn(move || { watch_status_file(sm_status_file_copy, guarded_queue_info)}); 

        let guarded_queue_info = self.status_info.clone();
        thread::spawn(move || { watch_queue_file(sm_queue_file_copy, guarded_queue_info)}); 
    }
}


fn update_from_queue_file(file_path : &path::Path, status_info: Arc<Mutex<QueueInfo>>) -> Result<()> {
    let mut contents = String::new();
    let mut file = File::open(file_path)?;
    file.read_to_string(&mut contents)?;
    
    let lines = contents.lines();
    let mut entries : Vec<QueueEntry> = vec![];
    for line in lines {
        if line.contains(";"){
            let types: Vec<&str> = line.splitn(4,';').collect(); // We expect 4 fields separated by ';'
            let id = match types[0].parse::<u64>() { Ok(value) => value, Err(error) => return Err(Error::new(ErrorKind::InvalidData, error))};
            let priority = match types[1].parse::<u64>(){ Ok(value) => value, Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Failed to entry priority"))};
            
            // Append new queue entry after parsing
            entries.push(QueueEntry::new(id, priority, types[2].to_string(), types[3].to_string()));
        }
    }

    // Assign new entries to status_info
    status_info.lock().unwrap().entry_list = entries;

    return Ok(());
}

fn update_from_status_file(file_path : &path::Path, status_info: Arc<Mutex<QueueInfo>>) -> Result<()> {
    let mut contents = String::new();
    let mut file = File::open(file_path)?;
    file.read_to_string(&mut contents)?;
    
    let mut lines = contents.lines();
    // Expected 2 (minimal) lines
    
    // Read playback time from the first line
    let playback_time: f64;
    match lines.next(){
        Some(value) => {
            let playbacktime_result = value.parse::<f64>();
            if playbacktime_result.is_err() {
                return Err(Error::new(ErrorKind::InvalidData, "Failed to cast playbacktime"));
            }
            playback_time = playbacktime_result.unwrap();
            
        }
        None => {
            return Err(Error::new(ErrorKind::InvalidData, "Playback time not found."));
        }
    }

    // 
    let playback_state : PlaybackState;
    match lines.next(){
        Some(value) => {
            match value {
                "Paused"    => {playback_state = PlaybackState::Paused}
                "Playing"   => {playback_state = PlaybackState::Playing}
                "Idle"      => {playback_state = PlaybackState::Idle}
                "Stopped"   => {playback_state = PlaybackState::Stopped}
                _           => {return Err(Error::new(ErrorKind::InvalidData, "Invalid state"));}
            }
        }
        None => {
            return Err(Error::new(ErrorKind::InvalidData, "Playback time not found."));
        }
    }

    // Assign to status info
    status_info.lock().unwrap().playback_time = playback_time;
    status_info.lock().unwrap().playback_state = playback_state;

    return Ok(());
}

fn watch_status_file(file_path_buf : path::PathBuf, status_info: Arc<Mutex<QueueInfo>>){
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let file_path = file_path_buf.as_path();

    // Create a watcher object, delivering raw events.
    // The notification back-end is selected based on the platform.
    let mut watcher = raw_watcher(tx).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    println!("Started watching file {:?}", file_path);
    match update_from_status_file(file_path, status_info.clone()) {Ok(_)=>{}, Err(error)=>println!("{}",error)};
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
           Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
                //println!("{:?} {:?} ({:?})", op, path, cookie);
                let result = match op {
                    notify::op::WRITE | notify::op::CREATE => {
                        let result = update_from_status_file(file_path, status_info.clone());
                        if result.is_err() {println!("{:?}", result);}
                        result
                    },
                    _ => { Ok(()) },
                };
                if let Err(e) = result {
                    println!("Got error trying to read file: {:?}", e); // #TODO: Store in struct (pass to terminal_ui)
                }
           },
           Ok(event) => println!("broken event: {:?}", event),
           Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn watch_queue_file(file_path_buf : path::PathBuf, status_info: Arc<Mutex<QueueInfo>>){
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let file_path = file_path_buf.as_path();

    // Create a watcher object, delivering raw events.
    // The notification back-end is selected based on the platform.
    let mut watcher = raw_watcher(tx).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    println!("Started watching file {:?}", file_path);
    match update_from_queue_file(file_path, status_info.clone()) {Ok(_)=>{}, Err(error)=>println!("{}",error)};
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
           Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
                //println!("{:?} {:?} ({:?})", op, path, cookie);
                let result = match op {
                    notify::op::WRITE | notify::op::CREATE => {
                        let result = update_from_queue_file(file_path, status_info.clone());
                        if result.is_err() {println!("{:?}", result);}
                        result
                    },
                    _ => { Ok(()) },
                };
                if let Err(e) = result {
                    println!("Got error trying to read file: {:?}", e); // #TODO: Store in struct (pass to terminal_ui)
                }
           },
           Ok(event) => println!("broken event: {:?}", event),
           Err(e) => println!("watch error: {:?}", e),
        }
    }
}