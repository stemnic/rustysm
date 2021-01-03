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

// File Parsing
use easy_reader::EasyReader;

// TIME
use chrono::prelude::DateTime;
use chrono::Utc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Debug, Clone)]
pub struct HistoryLogEntry {
    pub timestamp: String,
    pub name: String,
    pub location: String,
}

#[derive(Debug)]
pub struct HistoryWatcher {
    sm_history_file: path::PathBuf,
    pub entries: Vec<HistoryLogEntry>,
}

impl HistoryWatcher {
    pub fn new(history_file: path::PathBuf) -> Result<Self>{
        let stru = HistoryWatcher{
            sm_history_file: path::PathBuf::from(history_file),
            entries: Vec::new(),
        };
        return Ok(stru);
    }
    pub fn read(&mut self, num_lines_to_read: u64, line_offset_from_tail: u64) -> Result<()> {
        let file = File::open(self.sm_history_file.as_path())?;
        let mut reader = EasyReader::new(file)?;
        
        reader.eof();
        let mut lines_read = 0;
        let mut line_offset = 0;
        let mut entries = vec![];
        while let Some(line) = reader.prev_line()? {
            // Loop until we reach target start offset
            if (line_offset < line_offset_from_tail){
                line_offset += 1;
                continue;
            }

            let types: Vec<&str> = line.splitn(3,'\t').collect(); // We expect 3 fields separated by TAB
            if (types.len() < 3){
                continue;
            }

            let unix_timestamp_sec = match types[0].parse::<u64>() { Ok(value) => value, Err(error) => return Err(Error::new(ErrorKind::InvalidData, error))};
            // Create DateTime from SystemTime
            let datetime = DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(unix_timestamp_sec));
            // Formats the combined date and time with the specified format string.
            let timestamp_str = datetime.format("%H:%M %d-%b %y").to_string();

            // Append new history log entry after parsing
            entries.push(HistoryLogEntry{ timestamp : timestamp_str, name: types[1].to_string(), location: types[2].to_string()});
            
            lines_read += 1;
            if (lines_read >= num_lines_to_read){
                break;
            }
        }
        //println!("Read {:?} lines", lines_read);
        self.entries = entries;

        return Ok(())
    }
}
