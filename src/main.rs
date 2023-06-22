use yta_rs::{player_response, util};

#[tokio::main]
async fn main() {
    // Read url from args
    let url = std::env::args().nth(1).expect("No url provided");

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
        .get_dash_representations()
        .await
        .expect("Could not get download URLs");
    println!("Got download URLs: {:#?}", manifest);

    // Get the highest quality audio
    let mut audio = manifest
        .representations
        .iter()
        .filter(|r| r.height.is_none())
        .collect::<Vec<_>>();
    audio.sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth));
    let audio = audio.last().expect("Could not find audio representation");

    let mut video = manifest
        .representations
        .iter()
        .filter(|r| r.height.is_some())
        .collect::<Vec<_>>();
    video.sort_by(|a, b| a.bandwidth.cmp(&b.bandwidth));
    let video = video.last().expect("Could not find video representation");

    // Download the video
    for seq in 0..manifest.latest_segment_number {
        let audio_url = audio.get_url(seq);
        let video_url = video.get_url(seq);

        println!("Downloading audio segment {}", seq);
        util::download_file(
            audio_url.as_str(),
            format!("{}.f{}.ts", seq, audio.id).as_str(),
        )
        .await
        .expect("Could not download audio segment");

        println!("Downloading video segment {}", seq);
        util::download_file(
            video_url.as_str(),
            format!("{}.f{}.ts", seq, video.id).as_str(),
        )
        .await
        .expect("Could not download video segment");
    }
}
