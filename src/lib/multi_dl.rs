use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::{fs, io, thread};

use anyhow::{anyhow, Context, Result};
use console::style;
use curl::easy::{Easy2, Handler, WriteError};
use curl::multi::{Easy2Handle, Message, Multi};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{
    client::{self, user_agent},
    models::{Album, Track},
    utils::{make_path, tag_mp3, timestamp, track_path},
};

type Config<'a> = (&'a Track, PathBuf);
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

        let tf = track_format.map_or_else(String::new, std::clone::Clone::clone);

        let handles = dl
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, (track, root))| {
                if make_path(track, root, &tf).exists() {
                    eprintln!("`{}` already exist, skipping", track.name);
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
                let _ = message_handler(&message, &handles, &tf)
                    .context("Failed to process downloaded item(s)");
            });

            if still_alive {
                dl.client.wait(&mut [], Duration::from_secs(1))?;
            }
        }

        // dl.progress_meter.println("Downloaded")?;

        Ok(())
    }

    fn download(&'a self, token: usize, cfg: Config<'a>) -> Result<Easy2Handle<Collector>> {
        let pb = self.progress_meter.add(
            ProgressBar::new(0).with_style(
                ProgressStyle::with_template(
                    "{prefix} {msg}\n[{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
                )?
                .progress_chars("=> "),
            ),
        );

        let prefix = PathBuf::from(cfg.0.album.artist.clone())
            .join(cfg.0.album.album.clone())
            .join(cfg.0.name.clone());

        pb.set_prefix(prefix.display().to_string());

        let url = &cfg.0.url;
        let mut request = Easy2::new(Collector(Vec::new(), pb, cfg));

        request.url(&url[..])?;
        request.useragent(&user_agent())?;
        request.progress(true)?;

        let mut handle = self.client.add2(request)?;
        handle.set_token(token)?;

        Ok(handle)
    }
}

fn message_handler(
    message: &Message,
    handles: &HashMap<usize, Easy2Handle<Collector>>,
    track_fmt: &String,
) -> Result<()> {
    let token = message.token().map_err(|err| anyhow!("{err}"))?;
    let handle = &handles[&token];

    let Collector(buf, bar, (track, root)) = handle.get_ref();
    let Album {
        release_date,
        album_art_url,
        ..
    } = &track.album;

    match message
        .result_for2(handle)
        .expect("token mismatch with the `EasyHandle`")
    {
        Ok(()) => {
            bar.set_message("📥");
            let path = track_path(track, root, track_fmt)?;

            let mut file = fs::File::create(&path)?;

            io::copy(&mut buf.as_slice(), &mut file)?;

            bar.set_message("💾");

            let album_art_url = album_art_url.clone().unwrap_or_default();

            let album_art = if album_art_url.is_empty() {
                None
            } else if let Ok(album_art) = offload(album_art_url) {
                Some(album_art)
            } else {
                None
            };

            tag_mp3(album_art, timestamp(release_date), track, &path)?;

            bar.println(format!("{} {}", bar.prefix(), style("✔").green()));

            bar.finish_and_clear();
        }
        Err(error) => {
            println!("E: {} - <{}>", error, track.url);
        }
    }

    Ok(())
}

fn offload(url: String) -> Result<Vec<u8>> {
    let (tx, rx) = channel();

    thread::spawn(move || -> Result<()> {
        tx.send(client::get(&url)?)?;

        Ok(())
    });

    Ok(rx.recv()?)
}
