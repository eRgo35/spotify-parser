const fetch = require('isomorphic-unfetch')
const Spotify = require("spotify-url-info")
const {getData, getPreview} = Spotify(fetch)

let spotufy_regex = /https?:\/\/(?:embed\.|open\.)(?:spotify\.com\/)(?:track\/|\?uri=spotify:track:)((\w|-)+)(?:(?=\?)(?:[?&]foo=(\d*)(?=[&#]|$)|(?![?&]foo=)[^#])+)?(?=#|$)/
let isUrl = false
let url = ""

process.argv.forEach(val => {
  if (isUrl) {
    url = val
  }
  if (val == "--url") {
    isUrl = true
  }
})

let spotifyLink = spotufy_regex.test(url)
if (!spotifyLink) {
  console.error("invalid url")
  return
}

const parser = async () => {
  let spotifyResult = await getPreview(url)
  console.log(spotifyResult.artist, " - ", spotifyResult.title)
}

parser()
