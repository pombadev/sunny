use std::{env, path::PathBuf};

use clap::{Parser, ValueEnum};
use strfmt::strfmt;
use sunny::utils::format_container;

#[derive(Debug, Parser)]
#[clap(
    about,
    version,
    after_help = "Note: run --help to see full descriptions of each flags/options"
)]
pub struct Config {
    /// Artist's bandcamp username or full url
    #[clap(display_order = 1, value_parser, value_name = "ARTIST | URL")]
    pub(crate) url: String,

    /// Directory path where downloads should be saved to
    #[clap(short, long, display_order = 2, value_parser = validate_path, long_help = r"Directory path where downloads should be saved to.
By default files are saved in the current directory.")]
    pub(crate) path: Option<PathBuf>,

    /// Specify track format
    #[clap(
        short,
        long,
        value_parser = validate_format,
        value_name = "FORMAT",
        long_help = r"Specify track format: default is '{num} - {track}'

available keys:
    {num} - track number
    {track} - track
    {artist} - artist
    {album} - album

usage:
    -t='{num} - {track} - {album} {artist}'

expands to:
    2 - Track - Album Artist

note that `.mp3` is appended automatically.")]
    pub(crate) track_format: Option<String>,

    /// Skip downloading these albums, note that albums need to be delimited by ','
    /// eg: -s 'one,two' or --skip-albums=one,two
    #[clap(short = 'S', long, value_name = "ALBUMS", value_delimiter = ',')]
    pub(crate) skip_albums: Option<Vec<String>>,

    /// List albums/tracks available for download
    #[clap(short, long)]
    pub(crate) list_available: bool,

    /// Search artist, album, label, track or all, instead of downloading
    #[clap(short, long)]
    pub(crate) search: bool,

    /// Specify type to search for, available only for `--search` flag
    #[clap(long = "type", short = 'T', default_value_t = SearchType::Artists, requires = "search")]
    #[arg(value_enum)]
    pub(crate) r#type: SearchType,

    /// Do not do anything; just show what would happen
    #[clap(display_order = 1000, long)]
    pub(crate) dry_run: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum SearchType {
    All,
    Artists,
    Labels,
    Albums,
    Tracks,
}

impl SearchType {
    pub(crate) fn as_search_filter(&self) -> &str {
        match self {
            Self::All => "",
            Self::Artists => "b",
            Self::Labels => "b",
            Self::Albums => "a",
            Self::Tracks => "t",
        }
    }
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
