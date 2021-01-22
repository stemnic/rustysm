use std::io;
use alsa::mixer;
use alsa::Round;
use alsa::Ctl;
use std::io::{Error, ErrorKind};
use log::{error, info, warn, debug, trace};
use std::sync::mpsc::channel;

pub struct AlsaController {
    current_volume_db_percentage: f32,
    current_volume_db: f32,
    volume_max_db: f32,
    volume_min_db: f32,
    alsa_event_rx: std::sync::mpsc::Receiver<String>
}

fn get_normalized_volume(min: f32, max: f32, value: f32) -> f32{
    // Based upon https://github.com/alsa-project/alsa-utils/blob/9b02b42db48d4c202afcaf5001019534f30f6c96/alsamixer/volume_mapping.c#L83-L118
    // Does not work 100%, though well enough
    let normalized = f32::powf(10.0,(value - max) / 6000.0);
    let min_norm = f32::powf(10.0, (min - max) / 6000.0);
    let normalized = (normalized - min_norm) / (1.0 - min_norm);
    //let pos = min.to_db() - value.to_db();
    //normalized = 1.0 - f32::powf(2.0, (pos)/(8.0));
    return normalized
}

impl AlsaController 
{
    pub fn new() -> Result<Self, io::Error> {
        // Hmm init we do
            // create struct
            // update volume
        debug!("AlsaController Init");
        let (alsa_event_tx, alsa_event_rx) = channel();
        let mut sys_control = AlsaController { current_volume_db_percentage: 0.0, current_volume_db: 0.0, volume_max_db: 0.0, volume_min_db: 0.0, alsa_event_rx: alsa_event_rx };
        sys_control.update_volume().unwrap();
        std::thread::spawn(move || {
            let alsa_ctrl = Ctl::new("hw:0", false).unwrap();
            alsa_ctrl.subscribe_events(true).unwrap();
            loop{
                let event = alsa_ctrl.read().unwrap().unwrap();
                let elem_id = event.get_id();
                let elem_interface = elem_id.get_interface();
                match elem_id.get_name() {
                    Ok(name) => {
                        alsa_event_tx.send(name.to_string());
                        debug!("Got alsa card event {:?} {:?}", name, elem_interface);
                    }
                    Err(_) => {warn!("Alsa event error occured")}
                };

                
            }
            
        });
        Ok(sys_control)
    }

    fn update_volume_struct(&mut self, mixer_channel : &alsa::mixer::Selem) -> (){
        let (mixer_db_min, mixer_db_max) = mixer_channel.get_playback_db_range();
        //let (mixer_vol_min, mixer_vol_max) = mixer_channel.get_playback_volume_range(); // Returns weird and dumb alsa scaling. Dont use.
        self.volume_max_db = mixer_db_max.to_db();
        self.volume_min_db = mixer_db_min.to_db();
        self.current_volume_db = mixer_channel.get_playback_vol_db(mixer::SelemChannelId::Last).unwrap().to_db();
        self.current_volume_db_percentage = 1.0 - (self.current_volume_db / mixer_db_min.to_db());
        debug!("Read current alsa volume as {}dB ({}%)", self.current_volume_db, self.current_volume_db_percentage * 100.0);
    }

    pub fn wait_for_volume_event(&mut self) -> bool {
        match self.alsa_event_rx.try_recv() {
            Ok(event_string) => {
                match event_string.as_str() {
                    "Master Playback Volume" => true,
                    _ => false
                }
            },
            Err(_) => false
        }
    }

    pub fn update_volume(&mut self) -> Result<(), io::Error>{
        // Get handle to mixer channel
        let mixer = mixer::Mixer::new("default", true).unwrap();
        let mixer_select = mixer::SelemId::new("Master", 0);
        let mixer_channel = match mixer.find_selem(&mixer_select) {
            Some(value) => {value}
            None => {
                return Err(Error::new(ErrorKind::Other, "Failed to open alsa interface"));
            }
        };

        self.update_volume_struct(&mixer_channel);
        Ok(())
    }
    pub fn get_description_str(&self) -> String {
        return  self.volume_min_db.to_string() + "dB / " + &self.current_volume_db.to_string() + "dB / " + &self.volume_max_db.to_string() + "dB"
    }
    
    pub fn volume_increment_db(&mut self, num_steps : u32) -> Result<(), io::Error> {
        // Get handle to mixer channel
        let mixer = mixer::Mixer::new("default", true).unwrap();
        let mixer_select = mixer::SelemId::new("Master", 0);
        let mixer_channel = match mixer.find_selem(&mixer_select) {
            Some(value) => {value}
            None => {
                return Err(Error::new(ErrorKind::Other, "Failed to open alsa interface"));
            }
        };
        self.update_volume_struct(&mixer_channel);

        if self.current_volume_db_percentage < 1.0 {
            let to_db_part = ((1.0 - self.current_volume_db_percentage) - (0.01 * (num_steps as f32))) * self.volume_min_db;
            let to_db = mixer::MilliBel::from_db(to_db_part);
            mixer_channel.set_playback_db_all(to_db, Round::Floor).unwrap();
            debug!("Increasing volume from {}dB to {}dB", self.current_volume_db, to_db_part);
            self.update_volume_struct(&mixer_channel);
        }
        Ok(())
    }
    pub fn volume_decrement_db(&mut self, num_steps : u32) -> Result<(), io::Error> {
        // Get handle to mixer channel
        self.update_volume().unwrap();
        let mixer = mixer::Mixer::new("default", true).unwrap();
        let mixer_select = mixer::SelemId::new("Master", 0);
        let mixer_channel = match mixer.find_selem(&mixer_select) {
            Some(value) => {value}
            None => {
                return Err(Error::new(ErrorKind::Other, "Failed to open alsa interface"));
            }
        };
        self.update_volume_struct(&mixer_channel);

        if self.current_volume_db_percentage > 0.0 {
            let to_db_part = ((1.0 - self.current_volume_db_percentage) + (0.01 * (num_steps as f32))) * self.volume_min_db;
            let to_db = mixer::MilliBel::from_db(to_db_part);
            debug!("Decreasing volume from {}dB to {}dB", self.current_volume_db, to_db_part);
            mixer_channel.set_playback_db_all(to_db, Round::Floor).unwrap();
            self.update_volume_struct(&mixer_channel);
        }
        Ok(())
    }
    pub fn get_human_ear_volume_normalized(&mut self) -> f64 {
        get_normalized_volume(self.volume_min_db, self.volume_max_db, self.current_volume_db) as f64
    }
}