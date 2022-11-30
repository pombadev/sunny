mod cli;

use sunny::{
    multi_dl,
    spider::fetch_albums,
    utils::{prepare_directory, print_as_tree},
};

fn main() {
    if let Err(e) = app_main() {
        eprintln!("{e}")
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
        ..
    } = cli::Config::default();

    let albums = fetch_albums(&url)?;

    if list_available {
        print_as_tree(albums);
        return Ok(());
    }

    let tracks = albums
        .iter()
        .filter(|album| {
            if let Some(to_skip) = &skip_albums {
                !to_skip.contains(&album.album)
            } else {
                true
            }
        })
        .flat_map(|album| {
            let root =
                prepare_directory(path.as_ref(), album).expect("root directory to be created");

            album
                .tracks
                .iter()
                .map(move |track| (track, album, root.clone()))
        })
        .collect();

    multi_dl::Downloader::run(tracks, track_format.as_ref())?;

    Ok(())
}
