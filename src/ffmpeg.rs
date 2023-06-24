use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum FfmpegError {
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}

pub async fn mux(input: &Path, output: &Path) -> Result<(), FfmpegError> {
    let mut child = tokio::process::Command::new("ffmpeg");

    child
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(input)
        .arg("-c")
        .arg("copy")
        .arg(output);

    child.spawn().map_err(FfmpegError::IoError)?.wait().await?;

    Ok(())
}
