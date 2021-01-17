use std::io;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::fs::metadata;

// Logging
use log::{error, info, warn, debug, trace};

#[derive(Debug, Clone)]
enum MessageType {
    QueueEntryRequest = 0,
    QueueControlRequest
}

#[derive(Debug, Clone)]
enum ControlCommand {
    ClearQueue = 0,
    StopPlayback,
    PausePlayback,
    StartPlayback,
    SkipAndPlay,
    RemoveFromQueue,
    PromoteEntry,
}

#[derive(Debug, Clone)]
pub enum EntryType {
	YoutubeMedia = 0,
	FileStream,
	LocalMedia,
	Command
}



#[derive(Debug)]
struct Message {
    Type: MessageType,
    Priority: u64,
    Data: Vec<u8>
}

#[derive(Debug)]
pub struct SocketCom{

}
pub const DEFAULT_PRIORITY: u64 = 50;

impl SocketCom{
    pub fn new() -> Result<Self, io::Error> {
        let com = SocketCom{};
        Ok(com)
    }

    fn send_message(&mut self, msg : Message) -> Result<(), io::Error> {
        let mut stream = UnixStream::connect("/tmp/media_queue.sock")?;
        let mut tbs_msg: Vec<u8> = vec!();
        debug!("Sending unix socket message {:?}", msg);
        for byte in &msg.Priority.to_be_bytes(){
            tbs_msg.push(*byte);
        }
        trace!("Raw header {:?}", &msg.Priority.to_le_bytes());
        
        trace!("Raw Type {:?}", &msg.Type);
        tbs_msg.push(msg.Type as u8);
        

        for byte in &msg.Data{
            tbs_msg.push(*byte);
        }
        trace!("Raw Data {:?}", &msg.Data);

        trace!("Raw tbs_msg {:?}", &tbs_msg);
        stream.write_all(&tbs_msg)?;

        Ok(())
    }
    // Should not be used
    pub fn stop_playback(&mut self) -> Result<(), io::Error> {
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: vec![ControlCommand::StopPlayback as u8]};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn pause_playback(&mut self) -> Result<(), io::Error> {
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: vec![ControlCommand::PausePlayback as u8]};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn start_playback(&mut self) -> Result<(), io::Error> {
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: vec![ControlCommand::StartPlayback as u8]};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn skip_playback(&mut self) -> Result<(), io::Error> {
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: vec![ControlCommand::SkipAndPlay as u8]};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn promote_entry(&mut self, queue_id: u64) -> Result<(), io::Error> {
        debug!("Promoting entry id {}", queue_id);
        let mut tbs_data: Vec<u8> = vec![];
        tbs_data.push(ControlCommand::PromoteEntry as u8);
        for byte in &queue_id.to_le_bytes() {
            trace!("queue id {:?}", *byte);
            tbs_data.push(*byte);
        }
        trace!("promote_entry data block {:?}", tbs_data);
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: tbs_data};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn add_entry(&mut self, entry_type : EntryType, entry : String, priority : u64) -> Result<(), io::Error> {
        let mut tbs_data: Vec<u8> = vec![];
        let entry_clone = entry.clone();
        debug!("Adding entry {:?} {}", entry_type, entry);
        match entry_type {
            EntryType::LocalMedia => {
                tbs_data.push(EntryType::LocalMedia as u8);
                let md = metadata(entry).unwrap();
                if md.is_file() {
                    for byte in entry_clone.as_bytes() {
                        tbs_data.push(*byte);
                    }
                    let tbs_message = Message{Type: MessageType::QueueEntryRequest, Priority: priority, Data: tbs_data};
                    self.send_message(tbs_message)?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "md is not a file"))
                }
            }
            _ => {return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not supported entry type"))}
        }
        Ok(())
    }
}