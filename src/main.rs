mod cli;

use std::process::exit;

use sunny::{
    client,
    spider::{fetch_albums, search as Search},
    utils::{prepare_directory, print_as_tree},
};

fn main() {
    if let Err(e) = app_main() {
        eprintln!("Error: {e}");
        exit(1)
    }
}

fn parse_url(input: &str) -> anyhow::Result<String> {
    match url::Url::parse(input) {
        Ok(url) => Ok(url.into()),
        // assuming that user has passed just the artist name
        Err(err) => {
            if err == url::ParseError::RelativeUrlWithoutBase {
                return Ok(format!("https://{input}.bandcamp.com/music"));
            }

            Err(err.into())
        }
    }
}

fn app_main() -> anyhow::Result<()> {
    let cli::Config {
        path,
        url,
        track_format,
        // dry_run,
        skip_albums,
        list_available,
        search,
        r#type,
        ..
    } = cli::Config::default();

    if search {
        Search(&url, r#type.as_search_filter())?;

        return Ok(());
    }

    let url = parse_url(&url)?;

    let albums = fetch_albums(&url)?;

    if list_available {
        print_as_tree(&albums);
        return Ok(());
    }

    let tracks = albums
        .iter()
        .filter(|album| {
            skip_albums
                .as_ref()
                .map_or(true, |to_skip| !to_skip.contains(&album.album))
        })
        .flat_map(|album| {
            let root =
                prepare_directory(path.as_ref(), album).expect("root directory to be created");

            album.tracks.iter().map(move |track| (track, root.clone()))
        })
        .collect();

    client::MultiDownloader::run(tracks, track_format.as_ref())?;

    Ok(())
}
