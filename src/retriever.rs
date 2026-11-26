use std::sync::LazyLock;
use std::time::Duration;

use crate::error::ParseError;
use crate::parser::{parse_list, parse_single, SpotifyTrack, USER_AGENT};

/// A whole request (connect, headers and body) may take this long.
const TIMEOUT: Duration = Duration::from_secs(15);

fn embed_url(category: &str, id: &str) -> String {
    format!("https://embed.spotify.com/?uri=spotify:{category}:{id}")
}

/// The search queries in an embed page: one for a track, all of them for a list.
fn queries(category: &str, content: &str) -> Result<Vec<String>, ParseError> {
    let tracks = if category == "track" {
        vec![parse_single(content)?]
    } else {
        parse_list(content)?
    };
    Ok(tracks.iter().map(SpotifyTrack::query).collect())
}

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(TIMEOUT)
        .build()
        .expect("HTTP client builds")
});

static BLOCKING_CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
    reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(TIMEOUT)
        .build()
        .expect("HTTP client builds")
});

pub(crate) async fn retrieve_async(category: &str, id: &str) -> Result<Vec<String>, ParseError> {
    let content = CLIENT
        .get(embed_url(category, id))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    queries(category, &content)
}

pub(crate) fn retrieve(category: &str, id: &str) -> Result<Vec<String>, ParseError> {
    let content = BLOCKING_CLIENT
        .get(embed_url(category, id))
        .send()?
        .error_for_status()?
        .text()?;
    queries(category, &content)
}
