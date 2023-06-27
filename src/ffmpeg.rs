use std::path::{Path, PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum FfmpegError {
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}

pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<PathBuf>,
    pub date: Option<String>,
    pub video_id: Option<String>,
    pub faststart: bool,
}

pub async fn mux(input: &Path, metadata: &Metadata, output: &Path) -> Result<(), FfmpegError> {
    info!("Muxing {} to {}", input.display(), output.display());

    let mut child = tokio::process::Command::new("ffmpeg");

    child
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y");

    // Set input
    child.arg("-i").arg(input);

    // Add metadata
    if let Some(thumbnail) = &metadata.thumbnail {
        child
            .arg("-i")
            .arg(thumbnail)
            .arg("-map")
            .arg("0")
            .arg("-map")
            .arg("1");
    }
    if let Some(title) = &metadata.title {
        child.arg("-metadata").arg(format!("title={}", title));
    }
    if let Some(description) = &metadata.description {
        child
            .arg("-metadata")
            .arg(format!("description={}", description));
    }
    if let Some(date) = &metadata.date {
        child.arg("-metadata").arg(format!("date={}", date));
    }
    if let Some(video_id) = &metadata.video_id {
        child
            .arg("-metadata")
            .arg(format!("episode_id={}", video_id));
    }

    child.arg("-c").arg("copy");

    if metadata.thumbnail.is_some() {
        child.arg("-disposition:v:1").arg("attached_pic");
    }

    if metadata.faststart {
        child.arg("-movflags").arg("+faststart");
    }

    // Set output
    child.arg(output);

    child.spawn().map_err(FfmpegError::IoError)?.wait().await?;
    info!("Muxing complete");

    Ok(())
}
