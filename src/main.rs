use std::path::PathBuf;

use yta_rs::player_response;

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

    // Get the download URLs
    let download_urls = ipr.get_download_urls().await;
    println!("Got download URLs: {:#?}", download_urls);
}
