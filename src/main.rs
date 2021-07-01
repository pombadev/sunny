mod cli;
mod logger;

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::{sync::Arc, thread, time};

use logger::Logger;
use sunny::{
    error,
    models::{Album, Track},
    spider::fetch_albums,
    utils::{green_check, prepare_directory, timestamp, worker},
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
        ..
    } = cli::Config::default();

    let albums = fetch_albums(&url)?;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()?;

    let progress = MultiProgress::new();
    progress.set_draw_target(ProgressDrawTarget::stdout());

    let track_format = Arc::new(track_format);

    let total_albums = albums.len();

    albums
        .iter()
        .enumerate()
        .try_for_each(|(index, album)| -> error::Result<()> {
            prepare_directory(path.as_ref(), &album)?;

            let Album {
                ref album_art_url,
                ref album,
                ref artist,
                ref release_date,
                ref tags,
                ref tracks,
                ..
            } = album;

            let pb = progress.add(ProgressBar::new_spinner());
            pb.enable_steady_tick(100);
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{prefix} {spinner}\n ↳ {msg} ({elapsed})"),
            );

            let current_album = index + 1;
            let pre = Arc::new(format!("[{}/{}] {}", current_album, total_albums, album));
            let total_tracks = tracks.len();
            let release_date = timestamp(release_date);

            pb.set_prefix(&pre);

            tracks.par_iter().for_each(|track| {
                let album = Arc::new(album.clone());
                let album_art_url = Arc::new(album_art_url.clone());
                let artist = Arc::new(artist.clone());
                let path = Arc::new(path.clone());
                let pb = Arc::new(pb.clone());
                let tags = Arc::new(tags.clone());
                let track_format = Arc::clone(&track_format);
                let track = Track {
                    num: track.num,
                    name: track.name.clone(),
                    url: track.url.clone(),
                    lyrics: track.lyrics.clone(),
                };

                let pre = pre.clone();

                pool.spawn(move || {
                    let msg = format!("[{}/{}] {}", track.num, total_tracks, track.name.clone());

                    pb.set_message(&msg);

                    if dry_run {
                        pb.println(&format!(
                            "{}\n ↳ [{}/{}] {} {}",
                            pre,
                            track.num,
                            total_tracks,
                            track.name.clone(),
                            green_check("")
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
                            pb.println(&format!("{}\n ↳ {} {}", &pre, &msg, green_check("")));
                        }
                        Err(err) => {
                            if err == error::Error::FileExist(err.to_string()) {
                                Logger::info(err.to_string());
                            } else {
                                Logger::error(err.to_string());
                            }
                            pb.finish();
                        }
                    };
                });
            });

            // println!(
            //     "No of threads: {}",
            //     String::from_utf8_lossy(
            //         &*std::process::Command::new("sh")
            //             .arg("-c")
            //             .arg(format!("ps huH p {} |  wc -l", std::process::id()))
            //             .output()
            //             .expect("unable to run `ps`") // .stderr
            //             .stdout
            //     )
            //     .trim()
            // );

            Ok(())
        })?;

    progress.join()?;

    Ok(())
}
