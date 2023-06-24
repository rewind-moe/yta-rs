# yta-rs

Minimal implementation of
[Kethsar/ytarchive](https://github.com/Kethsar/ytarchive) in Rust.

## Usage

This crate is meant to be used as a library. Currently, the executable only has
one mode, which is to download the highest quality audio and video fragments,
and compose a HLS playlist.

```sh
# Start downloading
cargo run https://www.youtube.com/watch?v=Io7ucwiaONc

# Run a webserver
cd yta_dl
python3 -m http.server 8080
```
