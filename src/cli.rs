use std::path::PathBuf;

use clap::Parser;
use strfmt::strfmt;
use surf::Url;

use sunny::utils::format_container;

#[derive(Debug, Parser)]
#[clap(about, version, after_help = "note: run --help to see more details")]
pub struct Config {
    /// Artist's bandcamp username or full url
    #[clap(display_order = 1, parse(from_str = from_str), value_name = "ARTIST | URL")]
    pub(crate) url: String,

    /// Directory path where downloads should be saved to
    #[clap(short, long, display_order = 2, validator = validate_path, parse(from_str = expand_tilde), long_help = r"
Directory path where downloads should be saved to.
By default files are saved in the current directory.
")]
    pub(crate) path: Option<PathBuf>,

    /// Do not do anything; just show what would happen
    #[clap(long)]
    pub(crate) dry_run: bool,

    /// Specify track format
    #[clap(
        display_order = 100,
        short,
        long,
        validator = validate_format,
        value_name = "FORMAT",
        long_help = r"
Specify track format: default is '{num} - {track}'

available keys:
    {num} - track number
    {track} - track
    {artist} - artist
    {album} - album

usage:
    -t='{num} - {track} - {album} {artist}'

expands to:
    2 - ATrack - SomeAlbum SomeArtist

note that `.mp3` is appended automatically.
")]
    pub(crate) track_format: Option<String>,

    /// Skip downloading these albums, note that albums need to be delimited by ','
    /// eg: -s 'one,two' or --skip-albums='one,two'
    #[clap(
        short,
        long,
        multiple_values = true,
        value_name = "ALBUMS",
        value_delimiter = ',',
        require_delimiter = true
    )]
    pub(crate) skip_albums: Option<Vec<String>>,

    /// list albums/tracks available to download
    #[clap(short, long)]
    pub(crate) list_available: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config::parse()
    }
}

fn validate_format(f: &str) -> Result<String, String> {
    let vars = format_container(
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    );

    strfmt(f, &vars).map_err(|err| err.to_string())
}

fn from_str(input: &str) -> String {
    match Url::parse(input) {
        Ok(url) => url.to_string(),
        // assuming that user has passed just the artist name
        Err(err) => {
            if err.to_string() == "relative URL without a base" {
                return match Url::parse(&format!("https://{}.bandcamp.com/music", input)) {
                    Ok(u) => u.to_string(),
                    _ => input.to_string(),
                };
            }

            input.to_string()
        }
    }
}

pub fn expand_tilde(p: &str) -> PathBuf {
    if p.starts_with('~') {
        return PathBuf::from(p.replace('~', env!("HOME")));
    }

    PathBuf::from(p)
}

fn validate_path(path: &str) -> Result<PathBuf, String> {
    let path = expand_tilde(path);

    let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;

    if meta.is_dir() {
        Ok(path)
    } else {
        Err("No such directory".into())
    }
}
