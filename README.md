# RustyShowMovie (rustysm)
A terminal controlled mpv queue system. 

## Compile info
The rust compiler has gotten more strict about lifetimes since some of this program's dependencies (MPV dependency's depdencies) program was built. Thusly one needs to downgrade to an older version of the build system.
It has been tested and working on version `1.70.0`.
This can be done as follows:
```bash
rustup install 1.70.0
rustup override set 1.70.0
cargo build --release
```

## Requirements
- mpv
- alsa
- youtube-dl
- spotify/mpd (playback start and pause)
