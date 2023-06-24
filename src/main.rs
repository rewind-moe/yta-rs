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

    // Create a working directory
    let workdir = std::path::Path::new("yta_dl");
    if !workdir.exists() {
        tokio::fs::create_dir(workdir)
            .await
            .expect("Could not create working directory");
    }

    // Write the index.html file
    let index_path = workdir.join("index.html");
    let html = include_bytes!("../resources/index.html");
    tokio::fs::write(index_path, html)
        .await
        .expect("Could not write index.html");

    worker::start(&client, &ipr, workdir)
        .await
        .expect("Worker exited with error");

    println!("Done");
}
