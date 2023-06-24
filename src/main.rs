use yta_rs::{player_response::InitialPlayerResponse, util, worker};

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
    let ipr =
        InitialPlayerResponse::from_html(html.as_str()).expect("Could not parse player response");

    // Check if is live
    if ipr.is_usable() {
        println!("Video is live");
    } else {
        println!("Video is not live");
        return;
    }

    worker::start(&client, &ipr)
        .await
        .expect("Worker exited with error");

    println!("Done");
}
