mod cli;
mod logger;

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use std::{path::MAIN_SEPARATOR, sync::Arc, thread, time};

use logger::Logger;
use sunny::{
    error,
    models::{Album, Track},
    spider::fetch_albums,
    utils::{green_check, prepare_directory, print_as_tree, red_cross, timestamp, worker},
};

fn main() {
    if let Err(e) = app_main() {
        Logger::error(e.to_string());
    }
}

fn app_main() -> error::Result<()> {
    let cli::Config {
        path,
        url,
        track_format,
        dry_run,
        skip_albums,
        list_available,
        ..
    } = cli::Config::default();

    let thread_pool = rayon::ThreadPoolBuilder::new().build()?;

    let progress = MultiProgress::new();
    progress.set_draw_target(ProgressDrawTarget::stdout());
    let spinner_style = ProgressStyle::default_spinner().template("{prefix} {spinner} {wide_msg}");

    let albums = fetch_albums(&url)?;

    if list_available {
        print_as_tree(albums);
        return Ok(());
    }

    let track_format = Arc::new(track_format.unwrap_or_default());
    let total_albums = albums.len();

    albums
        .par_iter()
        .enumerate()
        .filter(|(_, album)| {
            if let Some(to_skip) = &skip_albums {
                !to_skip.contains(&album.album)
            } else {
                true
            }
        })
        .try_for_each(|(index, album)| -> error::Result<()> {
            prepare_directory(path.as_ref(), album)?;

            let Album {
                ref artist,
                ref album,
                ref release_date,
                ref tracks,
                ref tags,
                ref album_art_url,
                ..
            } = album;

            let current_album = index + 1;
            let prefix = Arc::new(format!(
                "[{}{MAIN_SEPARATOR}{}] {}",
                current_album, total_albums, &album
            ));
            let total_tracks = tracks.len();
            let album_prefix = Arc::new((&album).to_string());

            let pb = progress.add(ProgressBar::new_spinner());
            pb.enable_steady_tick(100);
            pb.set_prefix(prefix.to_string());

            let release_date = timestamp(release_date);

            tracks
                .par_iter()
                .try_for_each(|track| -> error::Result<()> {
                    let pb = pb.clone();
                    pb.set_style(spinner_style.clone());

                    let album = Arc::new(album.clone());
                    let album_art_url = Arc::new(album_art_url.clone().unwrap_or_default());
                    let artist = Arc::new(artist.clone());
                    let path = Arc::new(path.clone());
                    let pb = Arc::new(pb);
                    let tags = Arc::new(tags.clone().unwrap_or_default());
                    let track_format = Arc::clone(&track_format);
                    let track = Track { ..track.to_owned() };

                    let track_name = track.name.clone();
                    let prefix = album_prefix.clone();

                    thread_pool.spawn(move || {
                        pb.set_message(format!(
                            "[{}{MAIN_SEPARATOR}{}] {}",
                            track.num,
                            total_tracks,
                            track_name.clone()
                        ));

                        if dry_run {
                            pb.println(format!(
                                "{}{MAIN_SEPARATOR}{} {}",
                                &prefix.clone(),
                                &track_name.clone(),
                                style("✔").black().bold().dim()
                            ));
                            thread::sleep(time::Duration::from_millis(500));
                            return;
                        }

                        match worker(
                            album,
                            artist,
                            tags,
                            album_art_url,
                            release_date,
                            track,
                            path,
                            track_format,
                        ) {
                            Ok(_) => {
                                pb.println(format!(
                                    "{}{MAIN_SEPARATOR}{} {}",
                                    &prefix.clone(),
                                    &track_name.clone(),
                                    green_check()
                                ));
                            }
                            Err(err) => {
                                if err == error::Error::FileExist(err.to_string()) {
                                    Logger::info(err.to_string());
                                } else {
                                    Logger::error(err.to_string());
                                }
                                pb.println(format!(
                                    "{}{MAIN_SEPARATOR}{} {}",
                                    &prefix.clone(),
                                    &track_name.clone(),
                                    red_cross()
                                ));
                            }
                        };
                    });

                    Ok(())
                })?;

            Ok(())
        })?;

    progress.join()?;

    Ok(())
}
