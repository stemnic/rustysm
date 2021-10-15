use std::io;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::fs::metadata;
use std::fs;
use std::path::PathBuf;

use youtube_dl::{YoutubeDl, YoutubeDlOutput};

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
enum EntryType {
	YoutubeMedia = 0,
	FileStream,
	LocalMedia,
    Command,
    Unknown
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
        for byte in &msg.Priority.to_le_bytes(){
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
        info!("Stopping playback");
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: vec![ControlCommand::StopPlayback as u8]};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn pause_playback(&mut self) -> Result<(), io::Error> {
        info!("Pause playback");
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: vec![ControlCommand::PausePlayback as u8]};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn start_playback(&mut self) -> Result<(), io::Error> {
        info!("Start playback");
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: vec![ControlCommand::StartPlayback as u8]};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn skip_playback(&mut self) -> Result<(), io::Error> {
        info!("Skip playback");
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: vec![ControlCommand::SkipAndPlay as u8]};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn clear_queue(&mut self) -> Result<(), io::Error> {
        info!("Clear queue");
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: vec![ControlCommand::ClearQueue as u8]};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn delete_entry(&mut self, queue_id: u64) -> Result<(), io::Error> {
        info!("Deleting entry id {}", queue_id);
        let mut tbs_data: Vec<u8> = vec![];
        tbs_data.push(ControlCommand::RemoveFromQueue as u8);
        for byte in &queue_id.to_le_bytes() {
            trace!("queue id {:?}", *byte);
            tbs_data.push(*byte);
        }
        trace!("promote_entry data block {:?}", tbs_data);
        let tbs_message = Message{Type: MessageType::QueueControlRequest, Priority: DEFAULT_PRIORITY, Data: tbs_data};
        self.send_message(tbs_message)?;
        Ok(())
    }
    pub fn promote_entry(&mut self, queue_id: u64) -> Result<(), io::Error> {
        info!("Promoting entry id {}", queue_id);
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
    pub fn add_entry(&mut self, entry : String, priority : u64, raw : bool) -> Result<String, io::Error> {
        // Should do parsing to identify if it is a youtube video or not
        let entry_clone = entry.clone();
        let md = metadata(&entry_clone);
        let mut entry_type = EntryType::Unknown;
        let mut youtube_obj : Option<youtube_dl::YoutubeDlOutput> = None;
        if md.is_ok() {
            let meta_data = md.unwrap();
            if meta_data.is_file() || meta_data.is_dir(){
                entry_type = EntryType::LocalMedia;
            } else {
                debug!("This is not a path or direcotry {:?}", entry_clone);
            }
        } else {
            let is_youtube = YoutubeDl::new(&entry_clone)
                            .socket_timeout("5")
                            .run();
            if is_youtube.is_ok() {
                entry_type = EntryType::YoutubeMedia;
                youtube_obj = Some(is_youtube.unwrap());
            } else {
                println!("{:?}", is_youtube);
                debug!("Not recognized input file/url");
            }
        }
        if raw {
            // Forces filestream type
            entry_type = EntryType::FileStream;
        }
        info!("Adding entry {:?} {}", entry_type, entry);
        match entry_type {
            EntryType::LocalMedia => {
                let mut tbs_data: Vec<u8> = vec![];
                tbs_data.push(EntryType::LocalMedia as u8);
                let fullpath = fs::canonicalize(PathBuf::from(&entry_clone))?;
                let fullpath_string = fullpath.into_os_string().into_string().unwrap();
                for byte in fullpath_string.as_bytes() {
                    tbs_data.push(*byte);
                }
                let tbs_message = Message{Type: MessageType::QueueEntryRequest, Priority: priority, Data: tbs_data};
                self.send_message(tbs_message)?;
                Ok("Added ".to_string() + &fullpath_string)
            }
            EntryType::YoutubeMedia => {
                let video_object = youtube_obj.unwrap();
                let mut video_array: Vec<youtube_dl::SingleVideo> = vec![];
                match video_object {
                    YoutubeDlOutput::SingleVideo(value) => {
                        let video = *value;
                        video_array.push(video);
                        debug!("Youtube singel video object");
                    },
                    YoutubeDlOutput::Playlist(value) =>{
                        let playlist = *value;
                        for video in playlist.entries.unwrap() {
                            video_array.push(video);
                        }
                        debug!("Youtube playlist object");
                    }
                }
                let mut feedback_message: String = "".to_string();
                for video in video_array {
                    let mut tbs_data: Vec<u8> = vec![];
                    tbs_data.push(EntryType::YoutubeMedia as u8);
                    let mut tbs_id_string;
                    if video.extractor == Some("youtube".to_string()) {
                        tbs_id_string = (*video.id).to_string() + " - " + &(*video.title);
                    }else{
                        tbs_id_string = (*video.webpage_url.unwrap()).to_string() + " - " + &(*video.title);
                    }
                    debug!("Youtube video add {}", &tbs_id_string);
                    feedback_message = feedback_message + "Added Youtube video " + &(*video.title) + "\n";
                    for byte in tbs_id_string.as_bytes() {
                        tbs_data.push(*byte);
                    }
                    let tbs_message = Message{Type: MessageType::QueueEntryRequest, Priority: priority, Data: tbs_data};
                    self.send_message(tbs_message)?;
                }
                Ok(feedback_message)
            }
            EntryType::FileStream => {
                let mut tbs_data: Vec<u8> = vec![];
                tbs_data.push(EntryType::FileStream as u8);
                for byte in entry.as_bytes() {
                    tbs_data.push(*byte);
                }
                let tbs_message = Message{Type: MessageType::QueueEntryRequest, Priority: priority, Data: tbs_data};
                self.send_message(tbs_message)?;
                Ok("Pushed '".to_string() + &entry + "' as a filestream")
            }
            _ => {return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not supported entry type"))}
        }
    }
}
