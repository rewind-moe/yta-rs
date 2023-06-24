use futures::{stream::FuturesOrdered, try_join, StreamExt};

use crate::{dash, hls, player_response, util};

#[derive(thiserror::Error, Debug)]
pub enum WorkerError {
    #[error("Error getting initial player response")]
    InitialPlayerResponseError(#[from] player_response::PlayerResponseError),
    #[error("Could not find representation")]
    MissingRepresentation(String),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}

pub async fn start(
    client: &util::HttpClient,
    ipr: &player_response::InitialPlayerResponse,
) -> Result<(), WorkerError> {
    let manifest = ipr
        .get_dash_representations(&client)
        .await
        .map_err(WorkerError::InitialPlayerResponseError)?;

    let (tx_seq, rx_seq) = tokio::sync::mpsc::unbounded_channel();
    try_join!(
        thread_seq(&client, &tx_seq, &ipr),
        thread_download(&client, rx_seq, &manifest, 4)
    )
    .map(|_| ())
}

async fn thread_seq(
    client: &util::HttpClient,
    tx_seq: &tokio::sync::mpsc::UnboundedSender<i64>,
    ipr: &player_response::InitialPlayerResponse,
) -> Result<(), WorkerError> {
    let mut seq = 0;
    let mut last_seq_time = std::time::Instant::now();

    'out: loop {
        let manifest = ipr
            .get_dash_representations(&client)
            .await
            .map_err(WorkerError::InitialPlayerResponseError)?;

        if manifest.latest_segment_number > seq {
            for s in seq..manifest.latest_segment_number {
                if seq > 0 {
                    last_seq_time = std::time::Instant::now();
                    println!("Found new segment {}", s);
                }
                if tx_seq.send(s).is_err() {
                    println!("Failed to send segment number to download thread");
                    break 'out;
                }
            }
            seq = manifest.latest_segment_number;
        }

        if !ipr.is_usable() {
            println!("Video is no longer live");
            break;
        }

        if last_seq_time.elapsed().as_secs() > 30 {
            println!("No new segments found for 30 seconds, stopping");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    println!("Sequence thread exited");

    Ok(())
}

async fn thread_download(
    client: &util::HttpClient,
    rx_seq: tokio::sync::mpsc::UnboundedReceiver<i64>,
    manifest: &dash::Manifest,
    concurrency: usize,
) -> Result<(), WorkerError> {
    // Get the highest quality audio
    let mut audio = manifest
        .representations
        .iter()
        .filter(|r| r.height.is_none())
        .collect::<Vec<_>>();
    audio.sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth));
    let audio = *audio
        .last()
        .ok_or(WorkerError::MissingRepresentation("audio".to_string()))?;

    // Get the highest quality video
    let mut video = manifest
        .representations
        .iter()
        .filter(|r| r.height.is_some())
        .collect::<Vec<_>>();
    video.sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth));
    let video = *video
        .last()
        .ok_or(WorkerError::MissingRepresentation("video".to_string()))?;

    println!(
        "Video: {}x{} {}fps ({})",
        video.width.ok_or(WorkerError::MissingRepresentation(
            "video width".to_string()
        ))?,
        video.height.ok_or(WorkerError::MissingRepresentation(
            "video height".to_string()
        ))?,
        video.frame_rate.ok_or(WorkerError::MissingRepresentation(
            "video frame rate".to_string()
        ))?,
        video.codecs,
    );
    println!("Audio: {}kbps ({})", audio.bandwidth / 1000, audio.codecs);

    // Write the m3u8 file
    let segment_duration = std::time::Duration::from_millis(manifest.segment_duration as u64);
    let mut playlist = hls::IndexPlaylist::new("index.m3u8", &manifest, &audio, &video)
        .await
        .map_err(WorkerError::IoError)?;

    let mut tasks = FuturesOrdered::new();
    let mut seq_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx_seq);
    let mut is_done = false;

    loop {
        // Start new downloads if we have room
        while tasks.len() < concurrency && !is_done {
            match seq_stream.next().await {
                Some(seq) => {
                    tasks.push_back(util::download_av_segment(&client, &audio, &video, seq))
                }
                None => {
                    is_done = true;
                    break;
                }
            }
        }

        // Exit if there's nothing left to do
        if tasks.is_empty() && is_done {
            break;
        }

        // Write finished segments to playlist file
        match tasks.next().await {
            Some(Ok((fname_audio, fname_video))) => {
                playlist
                    .playlist_audio
                    .add_segment(&fname_audio, segment_duration)
                    .await
                    .map_err(WorkerError::IoError)?;
                playlist
                    .playlist_video
                    .add_segment(&fname_video, segment_duration)
                    .await
                    .map_err(WorkerError::IoError)?;
            }
            Some(Err(e)) => {
                println!("Could not download segment: {}", e);
            }
            None => (),
        }
    }

    Ok(())
}
