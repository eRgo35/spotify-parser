//! Track names (`Artist - Title`) from Spotify track, album and playlist URLs, read
//! from Spotify's public embed pages. No API key is needed.

mod error;
mod parser;
mod retriever;

use std::sync::LazyLock;

use regex::Regex;

pub use error::ParseError;

static SPOTIFY_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"https?://(?:embed\.|open\.)spotify\.com/(?:intl-[a-zA-Z-]+/)?(?:embed/)?(track|album|playlist)/([a-zA-Z0-9]+)",
    )
    .expect("valid regex")
});

/// The category (`track`, `album` or `playlist`) and ID in a Spotify URL.
fn parse_url(url: &str) -> Result<(&str, &str), ParseError> {
    let captures = SPOTIFY_URL.captures(url).ok_or(ParseError::InvalidUrl)?;
    let category = captures.get(1).ok_or(ParseError::InvalidUrl)?.as_str();
    let id = captures.get(2).ok_or(ParseError::InvalidUrl)?.as_str();
    Ok((category, id))
}

/// `Artist - Title` for each track behind a Spotify URL (blocking).
///
/// Anything that is not a Spotify URL gives [`ParseError::InvalidUrl`] without a
/// network request.
pub fn retrieve_url(url: &str) -> Result<Vec<String>, ParseError> {
    let (category, id) = parse_url(url)?;
    retriever::retrieve(category, id)
}

/// `Artist - Title` for each track behind a Spotify URL.
///
/// Anything that is not a Spotify URL gives [`ParseError::InvalidUrl`] without a
/// network request.
pub async fn retrieve_async_url(url: &str) -> Result<Vec<String>, ParseError> {
    let (category, id) = parse_url(url)?;
    retriever::retrieve_async(category, id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRACK: &str = "https://open.spotify.com/track/4PTG3Z6ehGkBFwjybzWkR8?si=e0a8c8ada8284e43";
    const MULTIPLE_ARTISTS_TRACK: &str =
        "https://open.spotify.com/track/1O0SdrryPeGp6eSSUibdgo?si=4ae58febe9e74eae";
    const PLAYLIST: &str =
        "https://open.spotify.com/playlist/37i9dQZF1DZ06evO05tE88?si=e0c6f44d176f44e6";
    const ALBUM: &str =
        "https://open.spotify.com/album/6eUW0wxWtzkFdaEFsTJto6?si=_grLtlNySNyfJTZr8tP44Q";

    #[test]
    fn url_forms() {
        assert_eq!(parse_url(TRACK), Ok(("track", "4PTG3Z6ehGkBFwjybzWkR8")));
        assert_eq!(
            parse_url("https://open.spotify.com/intl-pl/track/4PTG3Z6ehGkBFwjybzWkR8"),
            Ok(("track", "4PTG3Z6ehGkBFwjybzWkR8"))
        );
        assert_eq!(
            parse_url("https://open.spotify.com/intl-pt-BR/album/6eUW0wxWtzkFdaEFsTJto6"),
            Ok(("album", "6eUW0wxWtzkFdaEFsTJto6"))
        );
        assert_eq!(
            parse_url("https://open.spotify.com/embed/playlist/37i9dQZF1DZ06evO05tE88"),
            Ok(("playlist", "37i9dQZF1DZ06evO05tE88"))
        );
        for not_spotify in [
            "never gonna give you up",
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://open.spotify.com/artist/0gxyHStUsqpMadRV0Di1Qt",
        ] {
            assert_eq!(parse_url(not_spotify), Err(ParseError::InvalidUrl));
        }
    }

    #[tokio::test]
    async fn not_spotify_is_invalid_without_a_request() {
        assert_eq!(
            retrieve_async_url("just some words").await,
            Err(ParseError::InvalidUrl)
        );
    }

    // The tests below fetch from Spotify.

    #[test]
    fn check_track() {
        let track: Vec<&str> = vec!["Rick Astley - Never Gonna Give You Up"];
        assert_eq!(retrieve_url(TRACK).unwrap(), track);
    }

    #[test]
    fn check_multiple_artists_track() {
        let track: Vec<&str> = vec!["Will o' the wisp, Rick Astley - Blood on My Tie"];
        assert_eq!(retrieve_url(MULTIPLE_ARTISTS_TRACK).unwrap(), track);
    }

    #[tokio::test]
    async fn check_playlist() {
        // A generated playlist: its tracks change, so only the shape is checked.
        let tracks = retrieve_async_url(PLAYLIST).await.unwrap();
        assert!(tracks.len() >= 10, "{tracks:?}");
        assert!(tracks
            .iter()
            .all(|t| t.contains(" - ") && !t.contains("&nbsp;")));
        assert!(tracks.iter().any(|t| t.contains("Rick Astley")));
    }

    #[test]
    fn check_album() {
        let album: Vec<&str> = vec![
            "Rick Astley - Never Gonna Give You Up",
            "Rick Astley - Whenever You Need Somebody",
            "Rick Astley - Together Forever",
            "Rick Astley - It Would Take a Strong Strong Man",
            "Rick Astley - The Love Has Gone",
            "Rick Astley - Don't Say Goodbye",
            "Rick Astley - Slipping Away",
            "Rick Astley - No More Looking for Love",
            "Rick Astley - You Move Me",
            "Rick Astley - When I Fall in Love",
        ];
        assert_eq!(retrieve_url(ALBUM).unwrap(), album);
    }
}
