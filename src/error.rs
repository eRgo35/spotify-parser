use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// Not a Spotify track, album or playlist URL.
    InvalidUrl,
    /// The embed page could not be fetched.
    Request(String),
    /// The embed page did not contain the expected data.
    Parse(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidUrl => write!(f, "Invalid URL"),
            ParseError::Request(e) => write!(f, "Could not fetch the Spotify page: {e}"),
            ParseError::Parse(e) => write!(f, "Could not read the Spotify page: {e}"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<reqwest::Error> for ParseError {
    fn from(e: reqwest::Error) -> Self {
        ParseError::Request(e.to_string())
    }
}
