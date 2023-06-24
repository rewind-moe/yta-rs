use std::{path::Path, time::Duration};

use tokio::{
    fs::File,
    io::{self, AsyncWriteExt},
    try_join,
};

use crate::dash::{Manifest, Representation};

pub struct LivePlaylist {
    file: File,
}

impl LivePlaylist {
    pub async fn new(fname: &str, segment_duration: Duration) -> io::Result<Self> {
        let mut file = File::create(fname).await?;

        // Write the header
        file.write_all(
            format!(
                "#EXTM3U
#EXT-X-TARGETDURATION:{}
#EXT-X-MEDIA-SEQUENCE:{}
#EXT-X-VERSION:3\n",
                segment_duration.as_secs_f32(),
                0,
            )
            .as_bytes(),
        )
        .await?;

        Ok(Self { file })
    }

    pub async fn add_segment(&mut self, fname: &str, segment_duration: Duration) -> io::Result<()> {
        self.file
            .write_all(
                format!(
                    "#EXTINF:{:.1},\n{}\n",
                    segment_duration.as_secs_f32(),
                    fname
                )
                .as_bytes(),
            )
            .await
    }
}

pub struct IndexPlaylist {
    pub playlist_audio: LivePlaylist,
    pub playlist_video: LivePlaylist,
}

fn replace_extension(fname: &str, ext: &str) -> String {
    let path = Path::new(fname).to_path_buf();
    let path = path.with_extension(ext);
    String::from(path.to_string_lossy())
}

impl IndexPlaylist {
    pub async fn new(
        fname: &str,
        manifest: &Manifest,
        audio: &Representation,
        video: &Representation,
    ) -> io::Result<Self> {
        let mut file = File::create(fname).await?;

        let path_playlist_audio = replace_extension(fname, &format!("f{}.m3u8", audio.id));
        let path_playlist_video = replace_extension(fname, &format!("f{}.m3u8", video.id));
        let (fname_playlist_audio, fname_playlist_video) = (
            Path::new(&path_playlist_audio)
                .file_name()
                .expect("should never happen")
                .to_string_lossy(),
            Path::new(&path_playlist_video)
                .file_name()
                .expect("should never happen")
                .to_string_lossy(),
        );

        // Write the header
        file.write_all(
            format!(
                "#EXTM3U
#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"f{}\",DEFAULT=YES,AUTOSELECT=YES,URI=\"{}\"
#EXT-X-STREAM-INF:BANDWIDTH={},CODECS=\"{}\",AUDIO=\"f{}\"
{}\n",
                audio.id,
                fname_playlist_audio,
                video.bandwidth,
                video.codecs,
                audio.id,
                fname_playlist_video,
            )
            .as_bytes(),
        )
        .await?;

        let dur = Duration::from_millis(manifest.segment_duration as u64);
        let (playlist_audio, playlist_video) = try_join!(
            LivePlaylist::new(&path_playlist_audio, dur),
            LivePlaylist::new(&path_playlist_video, dur),
        )?;

        Ok(Self {
            playlist_audio,
            playlist_video,
        })
    }
}
