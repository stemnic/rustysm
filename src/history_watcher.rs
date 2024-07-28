// General
use std::io::Result;
use std::string::String;

// Threading
use std::sync::{Arc, Mutex};
use std::thread;

// File IO
use std::fs::File;
use std::path;

// File notification
use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};
use std::io::{Error, ErrorKind};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

// File Parsing
use easy_reader::EasyReader;

// TIME
use chrono::prelude::DateTime;
use chrono::Local;
use std::time::{Duration, UNIX_EPOCH};

// Logging
use log::{debug, warn};

pub const DEFAULT_HISTORY_ENTRIES_TO_FETCH: u64 = 100;

#[derive(Debug, Clone)]
pub struct HistoryLogEntry {
    pub timestamp: String,
    pub name: String,
    pub location: String,
}

#[derive(Debug)]
pub struct HistoryWatcher {
    sm_history_file: path::PathBuf,
    entries: Arc<Mutex<Vec<HistoryLogEntry>>>,
    lines_to_read: u64,
    line_offset: u64,
    history_update_rx: Receiver<()>,
    history_update_tx: Sender<()>,
}

fn read_history_file(
    file_path: &path::Path,
    num_lines_to_read: u64,
    line_offset_from_tail: u64,
    history_entries: Arc<Mutex<Vec<HistoryLogEntry>>>,
) -> Result<()> {
    let file = File::open(file_path)?;
    let mut reader = EasyReader::new(file)?;

    reader.eof();
    let mut lines_read = 0;
    let mut line_offset = 0;
    let mut entries = vec![];
    while let Some(line) = reader.prev_line()? {
        // Loop until we reach target start offset
        if line_offset < line_offset_from_tail {
            line_offset += 1;
            continue;
        }

        let types: Vec<&str> = line.splitn(3, '\t').collect(); // We expect 3 fields separated by TAB
        if types.len() < 3 {
            continue;
        }

        let unix_timestamp_sec = match types[0].parse::<u64>() {
            Ok(value) => value,
            Err(error) => return Err(Error::new(ErrorKind::InvalidData, error)),
        };
        // Create DateTime from SystemTime
        let datetime =
            DateTime::<Local>::from(UNIX_EPOCH + Duration::from_secs(unix_timestamp_sec));
        // Formats the combined date and time with the specified format string.
        let timestamp_str = datetime.format("%H:%M %d-%b %y").to_string();

        // Append new history log entry after parsing
        entries.push(HistoryLogEntry {
            timestamp: timestamp_str,
            name: types[1].to_string(),
            location: types[2].to_string(),
        });

        lines_read += 1;
        if lines_read >= num_lines_to_read {
            break;
        }
    }
    //println!("Read {:?} lines", lines_read);
    *history_entries.lock().unwrap() = entries;

    return Ok(());
}

impl HistoryWatcher {
    pub fn new(history_file: path::PathBuf, lines_to_read: u64, line_offset: u64) -> Result<Self> {
        let (tx, rx) = channel();
        let stru = HistoryWatcher {
            sm_history_file: path::PathBuf::from(history_file),
            entries: Arc::new(Mutex::new(Vec::new())),
            lines_to_read: lines_to_read,
            line_offset: line_offset,
            history_update_rx: rx,
            history_update_tx: tx,
        };
        read_history_file(
            &path::PathBuf::from(stru.sm_history_file.to_str().unwrap()),
            stru.lines_to_read,
            stru.line_offset,
            stru.entries.clone(),
        );
        return Ok(stru);
    }
    pub fn start(&mut self) {
        let history_file_copy = path::PathBuf::from(&self.sm_history_file);
        let guarded_history_entries = self.entries.clone();
        let history_update_tx = self.history_update_tx.clone();
        let lines_to_read = self.lines_to_read;
        let line_offset = self.line_offset;
        thread::spawn(move || {
            watch_history_file(
                history_file_copy,
                lines_to_read,
                line_offset,
                guarded_history_entries,
                history_update_tx,
            )
        });
    }
    pub fn get_history(&self) -> Vec<HistoryLogEntry> {
        (*self.entries.lock().unwrap()).clone()
    }

    pub fn check_for_status_change(&mut self) -> bool {
        match self.history_update_rx.try_recv() {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

fn watch_history_file(
    file_path_buf: path::PathBuf,
    num_lines_to_read: u64,
    line_offset_from_tail: u64,
    history_entries: Arc<Mutex<Vec<HistoryLogEntry>>>,
    update_notifier: Sender<()>,
) {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let file_path = file_path_buf.as_path();

    let mut watcher = raw_watcher(tx).unwrap();

    match read_history_file(
        file_path,
        num_lines_to_read,
        line_offset_from_tail,
        history_entries.clone(),
    ) {
        Ok(_) => {}
        Err(error) => warn!("{}", error),
    };
    watcher.watch(file_path, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(RawEvent {
                path: Some(path),
                op: Ok(op),
                cookie,
            }) => {
                //println!("{:?} {:?} ({:?})", op, path, cookie);
                let result = match op {
                    notify::op::WRITE | notify::op::CREATE => {
                        let result = read_history_file(
                            file_path,
                            num_lines_to_read,
                            line_offset_from_tail,
                            history_entries.clone(),
                        );
                        debug!("History file updated");
                        update_notifier.send(());
                        if result.is_err() {
                            warn!("{:?}", result);
                        }
                        result
                    }
                    _ => Ok(()),
                };
                if let Err(e) = result {
                    warn!("Got error trying to read file: {:?}", e); // #TODO: Store in struct (pass to terminal_ui)
                }
            }
            Ok(event) => warn!("broken event: {:?}", event),
            Err(e) => warn!("watch error: {:?}", e),
        }
    }
}
