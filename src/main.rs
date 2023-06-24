use tokio::{
    select,
    signal::unix::{signal, SignalKind},
};
use yta_rs::{player_response::InitialPlayerResponse, util, worker};

#[derive(thiserror::Error, Debug)]
enum RunError {
    #[error("Interrupted")]
    SignalInterrupt(#[from] std::io::Error),
    #[error("Worker error")]
    WorkerError(#[from] worker::WorkerError),
    #[error("Error")]
    Error(String, Box<dyn std::error::Error>),
}

async fn run(url: String) -> Result<(), RunError> {
    // Create HttpClient
    let client = util::HttpClient::new().expect("Could not create HttpClient");

    // Fetch the URL
    println!("Fetching {}", url);
    let html = reqwest::get(&url)
        .await
        .map_err(|e| RunError::Error("Could not fetch URL".to_string(), Box::new(e)))?
        .text()
        .await
        .map_err(|e| RunError::Error("Could not read response".to_string(), Box::new(e)))?;

    // Parse the HTML
    println!("Parsing initial player response");
    let ipr =
        InitialPlayerResponse::from_html(html.as_str()).expect("Could not parse player response");

    // Check if is live
    if ipr.is_usable() {
        println!("Video is live");
        ipr.video_details.as_ref().map(|v| {
            println!("[*] Title  : {}", v.title);
            println!("[*] Channel: {}", v.author);
        });
    } else {
        println!("Video is not live");
        return Ok(());
    }

    // Create a working directory
    let workdir = std::path::Path::new("yta_dl");
    if !workdir.exists() {
        tokio::fs::create_dir(workdir).await.map_err(|e| {
            RunError::Error(
                "Could not create working directory".to_string(),
                Box::new(e),
            )
        })?;
    }

    // Write the index.html file
    let index_path = workdir.join("index.html");
    let html = include_bytes!("../resources/index.html");
    tokio::fs::write(index_path, html)
        .await
        .map_err(|e| RunError::Error("Could not write index.html".to_string(), Box::new(e)))?;

    worker::start(&client, &ipr, workdir)
        .await
        .map_err(RunError::WorkerError)
}

#[tokio::main]
async fn main() {
    // Read url from args
    let url = std::env::args().nth(1).expect("No url provided");

    let (stop_tx, mut stop_rx) = tokio::sync::watch::channel(());

    let signal_process = tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        loop {
            select! {
                _ = sigterm.recv() => println!("Recieve SIGTERM"),
                _ = sigint.recv() => println!("Recieve SIGTERM"),
            };
            stop_tx.send(()).unwrap();
        }
    });

    loop {
        select! {
            biased;

            _ = stop_rx.changed() => {
                println!("Stop signal recieved");
                break;
            },
            res = run(url) => {
                println!("Worker process exited");
                if let Err(e) = res {
                    println!("Worker error: {:#?}", e);
                    std::process::exit(1);
                }
                break;
            },
            _ = signal_process => {
                println!("Signal process exited");
                std::process::exit(1);
            },
        }
    }
}
