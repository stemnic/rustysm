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


#[derive(Debug)]
struct QueueEntry{
    id: u64,
    priority: u64,
    entry_type: String,
    file_location: String, // Can be local filepath or url
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
    playback_status: String,
    playback_time: f64,
    entry_list: Vec<QueueEntry>,
}

impl QueueInfo {
    pub fn new() -> Self {
        let stru = QueueInfo{
            playback_status: String::from(""),
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
        let guarded_queue_info = self.status_info.clone();
        let sm_status_file_copy = path::PathBuf::from(&self.sm_status_file);
        thread::spawn(move || { watch_status_file(sm_status_file_copy, guarded_queue_info)}); 

        let guarded_queue_info = self.status_info.clone();
        let sm_queue_file_copy = path::PathBuf::from(&self.sm_queue_file);
        thread::spawn(move || { watch_queue_file(sm_queue_file_copy, guarded_queue_info)}); 
    }
}

fn watch_queue_file(file_path : path::PathBuf, status_info: Arc<Mutex<QueueInfo>>){
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering raw events.
    // The notification back-end is selected based on the platform.
    let mut watcher = raw_watcher(tx).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    println!("Started watching file {:?}", file_path);
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
           Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
               println!("{:?} {:?} ({:?})", op, path, cookie)
           },
           Ok(event) => println!("broken event: {:?}", event),
           Err(e) => println!("watch error: {:?}", e),
        }
    }
}


fn update_from_status_file(file_path : &path::Path, status_info: Arc<Mutex<QueueInfo>>) -> Result<()> {
    let mut contents = String::new();
    let mut file = File::open(file_path)?;
    file.read_to_string(&mut contents)?;

    // #TODO: finish this func. Read contents to struct

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
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
           Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
                println!("{:?} {:?} ({:?})", op, path, cookie);
                let result = match op {
                    notify::op::WRITE | notify::op::CREATE => update_from_status_file(file_path, status_info.clone()),
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