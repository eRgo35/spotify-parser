import fetch from 'isomorphic-unfetch'
import Spotify from 'spotify-url-info'

const {getData, getPreview} = Spotify(fetch)
const spotifyRegex = /https?:\/\/(?:embed\.|open\.)(?:spotify\.com\/)(?:track\/|\?uri=spotify:track:)((\w|-)+)(?:(?=\?)(?:[?&]foo=(\d*)(?=[&#]|$)|(?![?&]foo=)[^#])+)?(?=#|$)/
const spotifyPlaylistRegex = /https?:\/\/(?:embed\.|open\.)(?:spotify\.com\/)(?:(album|playlist)\/|\?uri=spotify:playlist:)((\w|-)+)(?:(?=\?)(?:[?&]foo=(\d*)(?=[&#]|$)|(?![?&]foo=)[^#])+)?(?=#|$)/

const parse = async url => {
  let isSpotifySong = spotifyRegex.test(url)
  let isSpotifyPlaylist = spotifyPlaylistRegex.test(url)
  
  if (isSpotifySong === isSpotifyPlaylist)
    return
  
  if (isSpotifyPlaylist) {
    let spotifyResultData = await getData(url).catch(() => null)
    if (!spotifyResultData || !['playlist', 'album'].includes(spotifyResultData.type))
      return
    
    for (const spotifyResult of spotifyResultData.trackList) {
      console.log(`${spotifyResult.subtitle} - ${spotifyResult.title}`)
    }

    return
  }

  let spotifyResult = await getPreview(url) 
  console.log(`${spotifyResult.artist} - ${spotifyResult.title}`)
}

let url = process.argv[process.argv.length - 1]
parse(url)
