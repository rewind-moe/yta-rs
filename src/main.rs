use futures::stream::StreamExt;
use tokio::try_join;
use yta_rs::{dash, player_response, util};

#[tokio::main]
async fn main() {
    // Read url from args
    let url = std::env::args().nth(1).expect("No url provided");

    // Create HttpClient
    let client = util::HttpClient::new().expect("Could not create HttpClient");

    // Fetch the URL
    println!("Fetching {}", url);
    let html = reqwest::get(&url)
        .await
        .expect("Could not fetch URL")
        .text()
        .await
        .expect("Could not read response body");

    // Parse the HTML
    println!("Parsing initial player response");
    let ipr = player_response::get_initial_player_response(html.as_str())
        .expect("Could not parse player response");
    println!("Got initial player response: {:#?}", ipr);

    // Check if is live
    if ipr.is_usable() {
        println!("Video is live");
    } else {
        println!("Video is not live");
    }

    // Get the download URLs
    let manifest = ipr
        .get_dash_representations(&client)
        .await
        .expect("Could not get download URLs");
    println!("Got download URLs: {:#?}", manifest);

    // Create queue for new segments
    let (tx_seq, rx_seq) = tokio::sync::mpsc::unbounded_channel();

    try_join!(
        thread_seq(tx_seq, &client, &ipr),
        thread_download(rx_seq, &client, &manifest, 4)
    )
    .expect("Tasks exited with error");
}

async fn thread_seq(
    tx_seq: tokio::sync::mpsc::UnboundedSender<i64>,
    client: &util::HttpClient,
    ipr: &player_response::InitialPlayerResponse,
) -> Result<(), ()> {
    let mut seq = 0;
    loop {
        let manifest = ipr
            .get_dash_representations(&client)
            .await
            .expect("Could not get download URLs");

        if manifest.latest_segment_number > seq {
            for s in seq..manifest.latest_segment_number {
                if seq > 0 {
                    println!("Found new segment {}", s);
                }
                tx_seq.send(s).unwrap();
            }
            seq = manifest.latest_segment_number;
        }

        if !ipr.is_usable() {
            println!("Video is no longer live");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Ok(())
}

async fn thread_download(
    rx_seq: tokio::sync::mpsc::UnboundedReceiver<i64>,
    client: &util::HttpClient,
    manifest: &dash::Manifest,
    concurrency: usize,
) -> Result<(), ()> {
    // Get the highest quality audio
    let mut audio = manifest
        .representations
        .iter()
        .filter(|r| r.height.is_none())
        .collect::<Vec<_>>();
    audio.sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth));
    let audio = *audio.last().expect("Could not find audio representation");

    // Get the highest quality video
    let mut video = manifest
        .representations
        .iter()
        .filter(|r| r.height.is_some())
        .collect::<Vec<_>>();
    video.sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth));
    let video = *video.last().expect("Could not find video representation");

    let rx_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx_seq);

    rx_stream
        .for_each_concurrent(concurrency, |seq| async move {
            let url_audio = audio.get_url(seq);
            let url_video = video.get_url(seq);
            println!("Downloading segment {}", seq);

            let fname = format!("seq_{:06}.ts", seq);
            util::download_and_merge_segment(&client, &url_audio, &url_video, &fname)
                .await
                .expect("Could not download segment");
        })
        .await;

    Ok(())
}
