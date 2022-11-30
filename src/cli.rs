use std::{env, path::PathBuf};

use clap::Parser;
use strfmt::strfmt;
use sunny::utils::format_container;
use url::Url;

#[derive(Debug, Parser)]
#[clap(about, version, after_help = "note: run --help to see more details")]
pub struct Config {
    /// Artist's bandcamp username or full url
    #[clap(display_order = 1, value_parser = parse_url, value_name = "ARTIST | URL")]
    pub(crate) url: String,

    /// Directory path where downloads should be saved to
    #[clap(short, long, display_order = 2, value_parser = validate_path, long_help = r"
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
        value_parser = validate_format,
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
    2 - Track - Album Artist

note that `.mp3` is appended automatically.
")]
    pub(crate) track_format: Option<String>,

    /// Skip downloading these albums, note that albums need to be delimited by ','
    /// eg: -s 'one,two' or --skip-albums=one,two
    #[clap(short, long, value_name = "ALBUMS", value_delimiter = ',')]
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
        &String::new(),
        &String::new(),
        &String::new(),
        &String::new(),
    );

    strfmt(f, &vars).map_err(|err| err.to_string())
}

fn parse_url(input: &str) -> Result<String, String> {
    match Url::parse(input) {
        Ok(url) => Ok(url.into()),
        // assuming that user has passed just the artist name
        Err(err) => {
            if err == url::ParseError::RelativeUrlWithoutBase {
                return Ok(format!("https://{input}.bandcamp.com/music"));
            }

            Err(err.to_string())
        }
    }
}

pub fn expand_tilde(p: &str) -> PathBuf {
    #[allow(deprecated)]
    let home = env::home_dir().expect("home_dir to exist");

    if p.starts_with('~') {
        return PathBuf::from(p.replace('~', &home.to_string_lossy()));
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
