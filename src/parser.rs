use scraper::{Html, Selector};
use serde_json::Value;

use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpotifyTrack {
    pub title: String,
    pub artist: String,
}

impl SpotifyTrack {
    /// `Artist - Title`, the form used as a search query.
    pub fn query(&self) -> String {
        format!("{} - {}", self.artist, self.title)
    }
}

pub(crate) const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

/// Spotify separates artists with no-break spaces; search engines want plain ones.
fn clean(text: &str) -> String {
    text.replace('\u{a0}', " ").trim().to_string()
}

/// The `entity` object of the embed page's `__NEXT_DATA__` JSON.
fn entity(content: &str) -> Result<Value, ParseError> {
    let document = Html::parse_document(content);
    let selector = Selector::parse("#__NEXT_DATA__").expect("valid selector");
    let script = document
        .select(&selector)
        .next()
        .ok_or_else(|| ParseError::Parse("no __NEXT_DATA__ element".into()))?;
    // The raw text: `inner_html` would re-escape characters such as U+00A0 as `&nbsp;`.
    let json: String = script.text().collect();
    let mut value: Value =
        serde_json::from_str(&json).map_err(|e| ParseError::Parse(e.to_string()))?;
    match value["props"]["pageProps"]["state"]["data"]["entity"].take() {
        Value::Null => Err(ParseError::Parse("no entity in the page data".into())),
        entity => Ok(entity),
    }
}

pub(crate) fn parse_single(content: &str) -> Result<SpotifyTrack, ParseError> {
    let entity = entity(content)?;
    let title = entity["title"]
        .as_str()
        .ok_or_else(|| ParseError::Parse("track has no title".into()))?;
    let artist = entity["artists"]
        .as_array()
        .map(|artists| {
            artists
                .iter()
                .filter_map(|artist| artist["name"].as_str())
                .map(clean)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    Ok(SpotifyTrack {
        title: clean(title),
        artist,
    })
}

pub(crate) fn parse_list(content: &str) -> Result<Vec<SpotifyTrack>, ParseError> {
    let entity = entity(content)?;
    let list = entity["trackList"]
        .as_array()
        .ok_or_else(|| ParseError::Parse("no track list".into()))?;
    // Entries without a title (e.g. removed tracks) are skipped, not fatal.
    Ok(list
        .iter()
        .filter_map(|track| {
            Some(SpotifyTrack {
                title: clean(track["title"].as_str()?),
                artist: clean(track["subtitle"].as_str().unwrap_or_default()),
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(entity: &str) -> String {
        format!(
            r#"<html><body><script id="__NEXT_DATA__" type="application/json">{{"props":{{"pageProps":{{"state":{{"data":{{"entity":{entity}}}}}}}}}}}</script></body></html>"#
        )
    }

    #[test]
    fn list_artists_use_plain_spaces() {
        let html = page(
            r#"{"trackList":[{"title":"Bring Back The Time","subtitle":"New Kids On The Block, Salt-N-Pepa"},{"subtitle":"no title"},{"title":"Solo","subtitle":"Me"}]}"#,
        );
        let tracks = parse_list(&html).unwrap();
        assert_eq!(
            tracks.iter().map(SpotifyTrack::query).collect::<Vec<_>>(),
            [
                "New Kids On The Block, Salt-N-Pepa - Bring Back The Time",
                "Me - Solo"
            ]
        );
    }

    #[test]
    fn single_joins_artists() {
        let html = page(
            r#"{"title":"Blood on My Tie","artists":[{"name":"Will o' the wisp"},{"name":"Rick Astley"}]}"#,
        );
        assert_eq!(
            parse_single(&html).unwrap().query(),
            "Will o' the wisp, Rick Astley - Blood on My Tie"
        );
    }

    #[test]
    fn missing_data_is_an_error_not_a_panic() {
        assert!(matches!(
            parse_list("<html></html>"),
            Err(ParseError::Parse(_))
        ));
        assert!(matches!(
            parse_single(&page("{}")),
            Err(ParseError::Parse(_))
        ));
        assert!(matches!(
            parse_list(&page(r#"{"title":"x"}"#)),
            Err(ParseError::Parse(_))
        ));
        let broken = r#"<script id="__NEXT_DATA__">{not json</script>"#;
        assert!(matches!(parse_single(broken), Err(ParseError::Parse(_))));
    }
}
