use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use std::{fs, io};

use anyhow::Result;
use console::style;
use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Multi};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{
    client::{self, user_agent},
    models::{Album, Track},
    utils::{file_path, make_path, tag_mp3, timestamp},
};

type Config<'a> = (&'a Track, &'a Album, PathBuf);
type RunConfig<'a> = Vec<Config<'a>>;

struct Collector<'a>(Vec<u8>, ProgressBar, Config<'a>);

impl Handler for Collector<'_> {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }

    fn progress(&mut self, dltotal: f64, dlnow: f64, _: f64, _: f64) -> bool {
        self.1.set_length(dltotal as u64);
        self.1.set_position(dlnow as u64);
        true
    }
}

pub struct Downloader<'a> {
    tracks: RunConfig<'a>,
    progress_meter: MultiProgress,
    client: Multi,
}

impl<'a> Downloader<'a> {
    pub fn run(dump: RunConfig<'a>, track_format: Option<&String>) -> Result<()> {
        let dl = Self {
            tracks: dump,
            progress_meter: MultiProgress::new(),
            client: Multi::new(),
        };

        let tf = if let Some(f) = track_format {
            f.to_owned()
        } else {
            String::new()
        };

        let handles = dl
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, (track, album, root))| {
                if make_path(&album.album, &album.artist, track, root, &tf).exists() {
                    eprintln!("`{}` already exist, skipping", track.name);
                    return false;
                }

                if track.url.is_empty() {
                    eprintln!("Track url is empty, skipping");
                    false
                } else {
                    true
                }
            })
            .map(|(token, track)| Ok((token, dl.download(token, track.clone())?)))
            .collect::<Result<HashMap<_, _>>>()?;

        let mut still_alive = true;

        while still_alive {
            if dl.client.perform()? == 0 {
                still_alive = false;
            }

            dl.client.messages(|message| {
                let token = message.token().expect("failed to get the token");
                let handle = &handles[&token];

                let Collector(buf, bar, (track, album, root)) = handle.get_ref();

                match message
                    .result_for2(handle)
                    .expect("token mismatch with the `EasyHandle`")
                {
                    Ok(()) => {
                        bar.set_message("📥");
                        let Ok(path) = file_path(&album.album, &album.artist, track, root, &tf) else {
                            // need to do something here
                            return;
                        };


                        let mut file = fs::File::create(&path).unwrap();

                        io::copy(&mut buf.as_slice(), &mut file).unwrap();

                        bar.set_message("💾");

                        let album_art_url = album.album_art_url.clone().unwrap_or_default();

                        let album_art = if album_art_url.is_empty() {
                            None
                        } else {
                            Some(client::get(&album_art_url).unwrap())
                        };

                        tag_mp3(
                            album,
                            album_art,
                            timestamp(&album.release_date),
                            track,
                            &path,
                        )
                        .unwrap();

                        bar.set_message("🎶");

                        bar.set_style(ProgressStyle::with_template("{msg}").unwrap());

                        bar.finish_with_message(format!("{} {}", track.name, style("✔").green()));
                    }
                    Err(error) => {
                        println!("E: {} - <{}>", error, track.url);
                    }
                }
            });

            if still_alive {
                dl.client.wait(&mut [], Duration::from_secs(1))?;
            }
        }

        // dl.progress_meter.println("Downloaded")?;

        Ok(())
    }

    fn download(&'a self, token: usize, track: Config<'a>) -> Result<Easy2Handle<Collector>> {
        let pb = self.progress_meter.add(
            ProgressBar::new(0).with_style(
                ProgressStyle::with_template(
                    &format!("{} [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} {{bytes_per_sec}} (eta {{eta}}){{msg}}", track.0.name),
                )?
                .progress_chars("#>-"),
            ),
        );
        let url = &track.0.url;
        let mut request = Easy2::new(Collector(Vec::new(), pb, track));

        request.url(&url[..])?;
        request.useragent(&user_agent())?;
        request.progress(true)?;

        let mut handle = self.client.add2(request)?;
        handle.set_token(token)?;

        Ok(handle)
    }
}
