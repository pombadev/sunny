use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_std::task;
use console::style;
use id3::{
    frame::{Lyrics, Picture, PictureType},
    Tag, TagLike, Timestamp, Version,
};
use strfmt::strfmt;
use surf::{middleware::Redirect, RequestBuilder};
use time::Date;

use crate::{
    error::{Error, Result},
    models::{Album, Track},
};

/// Apple Safari User Agent string
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Safari/605.1.15";

pub fn red_cross() -> String {
    style("✘").bold().red().to_string()
}

pub fn green_check() -> String {
    style("✔").bold().green().to_string()
}

pub fn client(url: impl AsRef<str>) -> RequestBuilder {
    surf::get(url)
        .header("User-Agent", USER_AGENT)
        .middleware(Redirect::default())
}

pub fn prepare_directory(path: Option<&PathBuf>, album: &Album) -> Result<()> {
    let path = match path {
        Some(path) => Path::new(path).join(&album.artist).join(&album.album),
        None => Path::new(&album.artist).join(&album.album),
    };

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    Ok(())
}

pub fn file_path(
    album: Arc<String>,
    artist: Arc<String>,
    track: &Track,
    path: Arc<Option<PathBuf>>,
    track_format: Arc<String>,
) -> Result<PathBuf> {
    let parent = Path::new(&artist.to_string()).join(&album.to_string());

    let file_name = if track_format.is_empty() {
        format!("{} - {}", &track.num, &track.name)
    } else {
        parse_track_template(
            &track_format,
            Arc::clone(&album),
            Arc::clone(&artist),
            track,
        )
    };

    let file = if let Some(ref path) = *path {
        path.join(parent.join(file_name)).with_extension("mp3")
    } else {
        parent.join(file_name).with_extension("mp3")
    };

    if file.exists() {
        return Err(Error::FileExist(file.display().to_string()));
    }

    Ok(file)
}

pub fn timestamp(date_string: &str) -> Option<Timestamp> {
    // if String looks like this `28 Sep 2014 04:19:31 GMT`
    match Date::parse(date_string, "%d %b %Y") {
        Ok(date) => Some(Timestamp {
            year: date.year(),
            month: Some(date.month()),
            day: Some(date.day()),
            hour: None,
            minute: None,
            second: None,
        }),
        Err(_) => {
            // if String looks like this `released September 28, 2014`
            if date_string.starts_with("released ") {
                let date_string = date_string.replace("released ", "");

                match Date::parse(date_string, "%d %b %Y") {
                    Ok(date) => Some(Timestamp {
                        year: date.year(),
                        month: Some(date.month()),
                        day: Some(date.day()),
                        hour: None,
                        minute: None,
                        second: None,
                    }),
                    Err(_) => None,
                }
            } else {
                None
            }
        }
    }
}

pub fn format_container(
    num: String,
    track: String,
    album: String,
    artist: String,
) -> HashMap<String, String> {
    HashMap::from([
        ("num".to_string(), num),
        ("track".to_string(), track),
        ("album".to_string(), album),
        ("artist".to_string(), artist),
    ])
}

pub fn parse_track_template(
    format: &str,
    album: Arc<String>,
    artist: Arc<String>,
    track: &Track,
) -> String {
    let Track { ref num, name, .. } = track;
    let vars = format_container(
        num.to_string(),
        String::from(name),
        album.to_string(),
        artist.to_string(),
    );

    // we can do this because we validate user's input
    strfmt(format, &vars).expect("failed to format keys")
}

#[allow(clippy::too_many_arguments)]
pub fn worker(
    album: Arc<String>,
    artist: Arc<String>,
    tags: Arc<String>,
    album_art_url: Arc<String>,
    release_date: Option<id3::Timestamp>,
    track: Track,
    path: Arc<Option<PathBuf>>,
    track_format: Arc<String>,
) -> Result<()> {
    let path = file_path(
        Arc::clone(&album),
        Arc::clone(&artist),
        &track,
        path,
        track_format,
    )?;

    if track.url.is_empty() {
        // eprintln!("Track url is empty, skipping");
        return Ok(());
    }

    let res = task::block_on(client(&track.url).recv_bytes())?;

    let mut file = task::block_on(async_std::fs::File::create(&path))?;

    task::block_on(async_std::io::copy(&mut res.as_slice(), &mut file))?;

    let album_art = if album_art_url.is_empty() {
        None
    } else {
        Some(task::block_on(
            client(album_art_url.to_string()).recv_bytes(),
        )?)
    };

    tag_mp3(album, artist, tags, album_art, release_date, &track, &path)?;

    Ok(())
}

pub fn tag_mp3(
    album: Arc<String>,
    artist: Arc<String>,
    tags: Arc<String>,
    album_art: Option<Vec<u8>>,
    release_date: Option<id3::Timestamp>,
    track: &Track,
    path: &Path,
) -> Result<()> {
    let mut tag = Tag::new();

    tag.set_title(&*track.name);
    tag.set_track(track.num as u32);
    tag.set_album(&*album);
    tag.set_artist(&*artist);
    tag.set_album_artist(&*artist);

    if let Some(ref lyrics) = track.lyrics {
        tag.add_frame(Lyrics {
            lang: "eng".to_string(),
            description: "".to_string(),
            text: String::from(lyrics.as_str()),
        });
    }

    if !tags.is_empty() {
        tag.set_genre(tags.to_string());
    }

    if let Some(album_art) = album_art {
        tag.add_frame(Picture {
            mime_type: "image/jpeg".to_string(),
            picture_type: PictureType::CoverFront,
            description: "".to_string(),
            data: album_art,
        });
    }

    if let Some(ts) = release_date {
        tag.set_date_recorded(ts);
    }

    tag.write_to_path(&path, Version::Id3v24)?;

    Ok(())
}

pub fn print_as_tree(albums: Vec<Album>) {
    if albums.is_empty() {
        println!("Noting to print");
    } else {
        let artist = &albums[0].artist;

        println!("{}", artist);

        for (album_index, album) in albums.iter().enumerate() {
            let next_album = albums.get(album_index + 1);

            let padding = if next_album.is_some() {
                "├──"
            } else {
                "└──"
            };

            println!("  {padding} {}", album.album);

            for (track_index, track) in album.tracks.iter().enumerate() {
                let next_track = album.tracks.get(track_index + 1);
                let bar = if next_album.is_some() { "│" } else { " " };

                let padding = if next_track.is_some() {
                    format!("  {bar}   ├──")
                } else {
                    format!("  {bar}   └──")
                };

                println!("{padding} {}", track.name);
            }
        }
    }
}
