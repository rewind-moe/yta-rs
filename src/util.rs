use std::{path::Path, sync::Arc};

use reqwest_cookie_store::CookieStoreMutex;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use tokio::{fs::File, io::AsyncWriteExt, try_join};

use crate::dash::Representation;

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

pub async fn download_av_segment(
    client: &HttpClient,
    outdir: &Path,
    audio: &Representation,
    video: &Representation,
    seq: i64,
) -> Result<(String, String), DownloadError> {
    let (url_audio, url_video) = (audio.get_url(seq), video.get_url(seq));
    let (fname_audio, fname_video) = (
        format!("seq_{:.6}.a{}.mp4", seq, audio.id),
        format!("seq_{:.6}.v{}.mp4", seq, video.id),
    );

    let dl_audio = async {
        let path_audio = outdir.join(&fname_audio);
        if let Ok(res) = tokio::fs::try_exists(&path_audio).await {
            if res {
                return Ok(());
            }
        }

        client
            .download_file(&url_audio, &path_audio.to_string_lossy())
            .await
    };
    let dl_video = async {
        let path_video = outdir.join(&fname_video);
        if let Ok(res) = tokio::fs::try_exists(&path_video).await {
            if res {
                return Ok(());
            }
        }

        client
            .download_file(&url_video, &path_video.to_string_lossy())
            .await
    };
    try_join!(dl_audio, dl_video)?;

    Ok((fname_audio, fname_video))
}
