# Spotify Parser

This crate allows to quickly ~~scrape~~ retrieve Spotify track title and artist from provided url.

```rust
let tracks = spotify_parser::retrieve_async_url(
    "https://open.spotify.com/album/6eUW0wxWtzkFdaEFsTJto6",
).await?;
// ["Rick Astley - Never Gonna Give You Up", ...]
```

Track, album and playlist URLs are supported (including `open.spotify.com/intl-xx/...`
links). Anything else returns `ParseError::InvalidUrl` without a network request;
network and page errors are returned as `ParseError::Request` / `ParseError::Parse`
instead of panicking.
