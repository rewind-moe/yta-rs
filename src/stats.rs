use std::io::Write;

use crate::util;

pub struct DownloadStatistics {
    pub bytes_downloaded: u64,
    pub segments_downloaded: u64,
    pub segments_total: u64,
}

impl DownloadStatistics {
    pub fn new() -> Self {
        Self {
            bytes_downloaded: 0,
            segments_downloaded: 0,
            segments_total: 0,
        }
    }

    pub fn print(&self) {
        print!(
            "\x1b[2K\rDownloaded {} of {} segments ({})",
            self.segments_downloaded,
            self.segments_total,
            util::format_bytes(self.bytes_downloaded)
        );
        let _ = std::io::stdout().lock().flush();
    }
}
