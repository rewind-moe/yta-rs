//! # yta-rs
//!
//! This crate provides a library for downloading YouTube DASH live streams. It
//! is based on [Kethsar/ytarchive](https://github.com/Kethsar/ytarchive), but
//! is more stripped down and geared towards being a library.
//!
//! ## Usage
//!
//! `yta-rs` is a low-level library, and so you'll need to write your own logic
//! to download and handle segments. The following example shows how to fetch
//! the initial player response and start downloading segments using the
//! `worker` module.
//!
//! ```rust
//! use yta_rs::{player_response::InitialPlayerResponse, util, worker};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create HttpClient, a wrapper around reqwest::Client but includes a
//!     // middleware for retrying transient errors
//!     let client = util::HttpClient::new().unwrap();
//!
//!     // Fetch the video page
//!     let html = client.fetch_text("https://www.youtube.com/watch?v=...").await.unwrap();
//!
//!     // Parse the initial player response
//!     let ipr = InitialPlayerResponse::from_html(html.as_str()).unwrap();
//!
//!     // Get the status of the stream
//!     if ipr.is_usable() {
//!         println!("Video is live");
//!     } else {
//!         println!("Video is not live");
//!         return;
//!     }
//!
//!     // Start the worker
//!     let workdir = std::path::Path::new(".");
//!     worker::start(&client, &ipr, workdir).await.unwrap();
//! }
//! ```
//!
//! The `worker` module provides a `start` function that will download segments
//! and write them to disk. It will also write an `index.m3u8` file that can be
//! used to play the stream.

#[forbid(unsafe_code)]
#[macro_use]
extern crate log;

pub mod dash;
pub mod ffmpeg;
pub mod hls;
pub mod player_response;
pub mod stats;
pub mod util;
pub mod worker;
