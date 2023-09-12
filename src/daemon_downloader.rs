use std::io;
use std::path;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::process::Command;
use std::fs;
use uuid::Uuid;

use youtube_dl::{YoutubeDl, YoutubeDlOutput};

use log::{error, info, warn, debug, trace};

#[derive(Debug)]
pub struct Downloader{
    finished: bool,
    failed: bool,
    url: String,
    download_directory: String,
    finished_notifier: Receiver<Vec<DownloadedObject>>,
    downloaded_objects: Vec<DownloadedObject>
}

#[derive(Debug, Clone)]
pub struct DownloadedObject {
    name: String,
    path: String,
}

impl Downloader {
    pub fn new(url:String, download_directory:String ) -> Result<Self, io::Error> {
        let (tx, rx) : (Sender<Vec<DownloadedObject>>, Receiver<Vec<DownloadedObject>>) = channel();
        let downloader = Downloader{ 
                                    finished:false, 
                                    failed:false, 
                                    url:url.clone(), 
                                    download_directory:download_directory.clone(), 
                                    finished_notifier:rx, 
                                    downloaded_objects: vec![] };
        std::thread::spawn(move || {
            let output = YoutubeDl::new(url.clone())
                .socket_timeout("15")
                .run();
            debug!("Output object {:?}", output);
            if output.is_ok(){
                let youtube_obj = output.unwrap();
                let mut video_array: Vec<youtube_dl::SingleVideo> = vec![];
                let mut is_playlist = false;
                match youtube_obj {
                    YoutubeDlOutput::SingleVideo(value) => {
                        let video = *value;
                        video_array.push(video);
                        debug!("Youtube singel video object");
                    },
                    YoutubeDlOutput::Playlist(value) =>{
                        let playlist = *value;
                        is_playlist = true;
                        for video in playlist.entries.unwrap() {
                            video_array.push(video);
                        }
                        debug!("Youtube playlist object");
                    }
                }

                let mut return_array = vec![];

                for video in video_array {
                    debug!("Downloading {}", &video.title.clone().expect("Could not extract video title"));
                    let download_path = path::Path::new(&download_directory);
                    let uuid_video = Uuid::new_v4();
                    let name = uuid_video.to_string();
                    let output = download_path.to_str().unwrap().to_string() + "/" + &name + ".%(ext)s";
                    let feedback = Command::new("yt-dlp")
                        .args(&["-o", &output, &video.webpage_url.unwrap(), "-i"])
                        .output()
                        .expect("yt-dlp command failed hard!");
                    let status_stdout = String::from_utf8_lossy(&feedback.stdout);
                    // Downloaddir/(original_queueid)-(arraypos).(format)
                    let mut resulting_video_path : String = "".to_string();
                    let paths = fs::read_dir(download_path).unwrap();
                    // Hacky way of finding the resulting filename
                    for path in paths {
                        let tmp_paht = path.unwrap().path();
                        let res_path = tmp_paht.to_str().unwrap();
                        if res_path.contains(&name) {
                            resulting_video_path = res_path.to_string();
                        }
                    }

                    let download_object = DownloadedObject{ name: video.title.clone().expect("Could not parse title"), path:resulting_video_path };
                    debug!("Resulting object {:?}", download_object);
                    return_array.push(download_object);

                }
                tx.send(return_array).unwrap();
            } else {
                // Check and download other types of media
                warn!("yt-dlp did not like {:?}", url);
                tx.send(vec![]).unwrap();
            }
        });

        Ok(downloader)
    }

    pub fn check_download_ready(&mut self) -> Option<Vec<DownloadedObject>> {
        match self.finished_notifier.try_recv(){
            Ok(value) => {
                self.downloaded_objects = value.clone();
                self.finished = true;
                self.failed = false;
                Some(value)
            }
            Err(_) => {
                None
            }
        }
    }

    pub fn cleanup_downloaded_videoes(&self) -> () {
        for video in self.downloaded_objects.clone() {
            fs::remove_file(video.path).unwrap();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Config, Logger, Root};

    fn init_log() {
        let stdout = ConsoleAppender::builder().build();

        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(Root::builder().appender("stdout").build(LevelFilter::Trace))
            .unwrap();
    
        let handle = log4rs::init_config(config).unwrap();
    }

    #[test]
    fn test_single_video() {
        init_log();
        let mut download = Downloader::new("https://www.youtube.com/watch?v=138ajKRMzIY".to_string(), "/tmp/".to_string()).unwrap();
        let mut result = vec![];
        loop{
            let res = download.check_download_ready();
            if res.is_some(){
                result = res.unwrap();
                break;
            }
            std::thread::sleep_ms(100);
        }
        println!("{:?}", result);
        assert_eq!(result.len(), 1);
        download.cleanup_downloaded_videoes();
        for video in result {
            assert_eq!(path::Path::new(&video.path).exists(), false);
        }
    }

    #[test]
    fn test_playlist_video() {
        init_log();
        // Needs a playlist of stable videos
        let mut download = Downloader::new("https://youtube.com/playlist?list=PL-OLcteU63B5IiWt-aCICEHH2fjLnvGbA".to_string(), "/tmp/".to_string()).unwrap();
        let mut result = vec![];
        loop{
            let res = download.check_download_ready();
            if res.is_some(){
                result = res.unwrap();
                break;
            }
            std::thread::sleep_ms(100);
        }
        println!("{:?}", result);
        assert_eq!(result.len(), 22);
        download.cleanup_downloaded_videoes();
        for video in result {
            assert_eq!(path::Path::new(&video.path).exists(), false);
        }
    }
}
