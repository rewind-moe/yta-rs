use tokio::{fs::File, io::AsyncWriteExt};

#[derive(thiserror::Error, Debug)]
pub enum DownloadError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub async fn download_file(url: &str, path: &str) -> Result<(), DownloadError> {
    let temp_path = format!("{}.tmp", path);
    let mut file = File::create(&temp_path).await?;
    let mut resp = reqwest::get(url).await?;

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
    }

    file.flush().await?;
    std::fs::rename(temp_path, path)?;

    Ok(())
}
