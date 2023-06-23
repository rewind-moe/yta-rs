use std::sync::Arc;

use reqwest_cookie_store::CookieStoreMutex;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use tokio::{fs::File, io::AsyncWriteExt, try_join};

pub struct HttpClient {
    pub client: ClientWithMiddleware,
    pub cookies: Arc<CookieStoreMutex>,
}

#[derive(thiserror::Error, Debug)]
pub enum DownloadError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("reqwest middleware error: {0}")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

impl HttpClient {
    pub fn new() -> reqwest::Result<HttpClient> {
        let cookies = Arc::new(CookieStoreMutex::default());
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = reqwest::Client::builder()
            .cookie_provider(cookies.clone())
            .build()?;

        let client = reqwest_middleware::ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(HttpClient { client, cookies })
    }

    pub async fn download_file(&self, url: &str, path: &str) -> Result<(), DownloadError> {
        let temp_path = format!("{}.tmp", path);
        let mut file = File::create(&temp_path).await?;
        let mut resp = self.client.get(url).send().await?;

        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        std::fs::rename(temp_path, path)?;

        Ok(())
    }

    pub async fn fetch_text(&self, url: &str) -> Result<String, DownloadError> {
        self.client
            .get(url)
            .send()
            .await?
            .text()
            .await
            .map_err(|e| e.into())
    }
}

pub async fn download_and_merge_segment(
    client: &HttpClient,
    url_audio: &str,
    url_video: &str,
    path: &str,
) -> Result<(), DownloadError> {
    let fname_audio = format!("{}.audio.fmp4", path);
    let fname_video = format!("{}.video.fmp4", path);

    try_join!(
        client.download_file(url_audio, &fname_audio),
        client.download_file(url_video, &fname_video),
    )?;

    let mut ffmpeg = tokio::process::Command::new("ffmpeg");
    ffmpeg
        .arg("-hide_banner")
        .arg("-y")
        .arg("-i")
        .arg(&fname_audio)
        .arg("-i")
        .arg(&fname_video)
        .arg("-c")
        .arg("copy")
        .arg("-f")
        .arg("mpegts")
        .arg(path);
    ffmpeg.stdin(std::process::Stdio::null());

    let out = ffmpeg.output().await?;
    if !out.status.success() {
        return Err(DownloadError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("ffmpeg failed: {:?}", out),
        )));
    }

    try_join!(
        tokio::fs::remove_file(&fname_audio),
        tokio::fs::remove_file(&fname_video),
    )?;

    Ok(())
}
