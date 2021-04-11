use std::process::Command;

pub fn spotify_playing() -> bool {
    let status = Command::new("dbus-send")
        .args(&["--print-reply","--session","--dest=org.mpris.MediaPlayer2.spotify","/org/mpris/MediaPlayer2", "org.freedesktop.DBus.Properties.Get", "string:org.mpris.MediaPlayer2.Player","string:PlaybackStatus"])
        .output()
        .expect("dbus command failed hard!");
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    if (*status_stdout).contains("Playing")  {
        return true;
    }
    return false;
}

pub fn spotify_play_pause() -> () {
    Command::new("dbus-send")
        .args(&["--print-reply","--dest=org.mpris.MediaPlayer2.spotify","/org/mpris/MediaPlayer2","org.mpris.MediaPlayer2.Player.PlayPause"])
        .spawn()
        .expect("dbus command failed hard!");
}

pub fn mpd_playing() -> bool {
    let status = match Command::new("mpc")
    .args(&["status"])
    .output() {
        Ok(value) => value,
        Err(error) => return false,
    };
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    if (*status_stdout).contains("playing")  {
        return true;
    }
    return false;
}

pub fn mpd_play() -> () {
    Command::new("mpc")
    .args(&["play"])
    .spawn()
    .ok();
}

pub fn mpd_pause() -> () {
    Command::new("mpc")
    .args(&["pause"])
    .spawn()
    .ok();
}

pub fn mpd_play_pause() -> () {
    if mpd_playing() {
        mpd_pause();
    } else {
        mpd_play();
    }
}