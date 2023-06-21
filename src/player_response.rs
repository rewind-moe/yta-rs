use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_aux::prelude::*;

// Generated with https://transform.tools/json-to-rust-serde

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialPlayerResponse {
    pub response_context: ResponseContext,
    pub playability_status: PlayabilityStatus,
    pub streaming_data: Option<StreamingData>,
    pub video_details: VideoDetails,
    pub microformat: Microformat,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseContext {
    pub main_app_web_response_context: MainAppWebResponseContext,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MainAppWebResponseContext {
    pub logged_out: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayabilityStatus {
    pub status: Status,
    pub reason: Option<String>,
    pub live_streamability: Option<LiveStreamability>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Ok,
    LiveStreamOffline,
    Unplayable,
    Error,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStreamability {
    pub live_streamability_renderer: LiveStreamabilityRenderer,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStreamabilityRenderer {
    pub video_id: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub poll_delay_ms: i64,
    pub offline_slate: Option<OfflineSlate>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineSlate {
    pub live_stream_offline_slate_renderer: LiveStreamOfflineSlateRenderer,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStreamOfflineSlateRenderer {
    #[serde(deserialize_with = "deserialize_datetime_utc_from_seconds")]
    pub scheduled_start_time: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingData {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub expires_in_seconds: i64,
    pub adaptive_formats: Vec<AdaptiveFormat>,
    pub dash_manifest_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptiveFormat {
    pub itag: i64,
    pub url: String,
    pub mime_type: String,
    pub bitrate: i64,
    pub target_duration_sec: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetails {
    pub video_id: String,
    pub title: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub length_seconds: i64,
    #[serde(default)]
    pub is_live: bool,
    pub channel_id: String,
    pub is_owner_viewing: bool,
    pub short_description: String,
    pub allow_ratings: bool,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub view_count: i64,
    pub author: String,
    pub is_live_content: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Microformat {
    pub player_microformat_renderer: PlayerMicroformatRenderer,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMicroformatRenderer {
    pub thumbnail: Thumbnail,
    pub owner_profile_url: String,
    pub publish_date: String,
    pub live_broadcast_details: LiveBroadcastDetails,
    pub upload_date: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub thumbnails: Vec<ThumbnailURL>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailURL {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveBroadcastDetails {
    pub is_live_now: bool,
    pub start_timestamp: String,
}

#[derive(thiserror::Error, Debug)]
pub enum PlayerResponseError {
    #[error("Could not find initial player response")]
    NoInitialPlayerResponse,
    #[error("Could not parse initial player response")]
    ParseInitialPlayerResponse(#[from] serde_json::Error),
}

const IPR_STR: &str = "var ytInitialPlayerResponse =";

fn get_ipr_str(html: &str) -> Option<&str> {
    // Find the start of the initial player response
    let idx_ipr = html.find(IPR_STR)? + IPR_STR.len();

    // Find the start and end of the JSON object
    let idx_start = html[idx_ipr..].find("{")? + idx_ipr;
    let idx_end = html[idx_start..].find("};")? + idx_start + 1;

    // Bounds check
    if idx_start >= idx_end || idx_start >= html.len() || idx_end >= html.len() {
        return None;
    }

    Some(&html[idx_start..idx_end])
}

pub fn get_initial_player_response(
    html: &str,
) -> Result<InitialPlayerResponse, PlayerResponseError> {
    // Find the initial player response
    let ipr_str = get_ipr_str(html).ok_or(PlayerResponseError::NoInitialPlayerResponse)?;

    // Parse the JSON
    serde_json::from_str(ipr_str).map_err(PlayerResponseError::ParseInitialPlayerResponse)
}

impl InitialPlayerResponse {
    pub fn is_usable(&self) -> bool {
        self.video_details.video_id != ""
            && self
                .playability_status
                .live_streamability
                .as_ref()
                .map(|ls| ls.live_streamability_renderer.video_id != "")
                .unwrap_or(false)
            && self.playability_status.status == Status::Ok
            && self
                .microformat
                .player_microformat_renderer
                .live_broadcast_details
                .is_live_now
    }

    pub fn target_duration(&self) -> Option<f64> {
        self.streaming_data
            .as_ref()?
            .adaptive_formats
            .first()?
            .target_duration_sec
    }

    async fn fetch_text(url: &str) -> Result<String, reqwest::Error> {
        reqwest::get(url).await?.text().await
    }

    pub async fn get_download_urls(&self) -> HashMap<i64, String> {
        let mut urls = self
            .streaming_data
            .as_ref()
            .map(|sd| {
                sd.adaptive_formats
                    .iter()
                    .map(|af| (af.itag, af.url.clone()))
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();

        // Download the DASH manifest if it exists
        if let Some(dashUrl) = self
            .streaming_data
            .as_ref()
            .and_then(|sd| sd.dash_manifest_url.as_ref())
        {
            let text = Self::fetch_text(dashUrl).await.unwrap();
        }

        urls
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn ipr_str() {
        let test_str = r#"<script>var ytInitialPlayerResponse = {"response": "test"};</script>"#;
        let result = get_ipr_str(test_str).expect("Could not find IPR");
        assert_eq!(result, r#"{"response": "test"}"#);

        let test_str = r#"<script>var ytInitialPlayerResponse = {"#;
        assert!(get_ipr_str(test_str).is_none());

        let test_str = r#"<script>var ytInitialPlayerResponse = "#;
        assert!(get_ipr_str(test_str).is_none());

        let test_str = r#"<script>var ytInitialPlayerResponse ="#;
        assert!(get_ipr_str(test_str).is_none());
    }

    fn get_test_html(fname: &str) -> String {
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/");
        d.push(fname);
        std::fs::read_to_string(d).expect(format!("Could not read {}", fname).as_str())
    }

    #[test]
    fn ipr_live() {
        let html = get_test_html("watchpage_live.html");
        let ipr = get_initial_player_response(&html).expect("Could not parse IPR");

        assert_eq!(ipr.video_details.is_live, true, "Video is not live");
        assert_eq!(ipr.video_details.length_seconds, 0, "Video length is not 0");
        assert_eq!(
            ipr.video_details.view_count, 210_943_922,
            "View count is not correct"
        );
        assert!(
            ipr.playability_status
                .live_streamability
                .expect("No live streamability")
                .live_streamability_renderer
                .offline_slate
                .is_none(),
            "Video is not livestreamable"
        );
    }

    #[test]
    fn ipr_scheduled() {
        let html = get_test_html("watchpage_scheduled.html");
        let ipr = get_initial_player_response(&html).expect("Could not parse IPR");

        assert_eq!(ipr.video_details.is_live, false, "Video is live");
        assert_eq!(
            ipr.playability_status.status,
            Status::LiveStreamOffline,
            "Playability status is not LiveStreamOffline"
        );
        assert_eq!(ipr.video_details.length_seconds, 0, "Video length is not 0");
        assert_eq!(ipr.video_details.view_count, 0, "View count is not correct");
        assert_eq!(
            ipr.playability_status
                .live_streamability
                .expect("No live streamability")
                .live_streamability_renderer
                .offline_slate
                .expect("Video should be offline")
                .live_stream_offline_slate_renderer
                .scheduled_start_time,
            DateTime::<Utc>::from_str("2024-02-15T08:15:00Z").unwrap(),
            "Video schedule does not match"
        );
    }
}
