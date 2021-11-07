use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_std::task;
use console::Style;
use id3::{
    frame::{Lyrics, Picture, PictureType},
    Tag, Timestamp, Version,
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

pub fn client(url: impl AsRef<str>) -> RequestBuilder {
    surf::get(url)
        .header("User-Agent", USER_AGENT)
        .middleware(Redirect::default())
}

pub fn prepare_directory(path: Option<&PathBuf>, album: &Album) -> Result<()> {
    let path = match path {
        Some(path) => format!("{}/{}/{}", path.display(), &album.artist, &album.album),
        None => format!("{}/{}", &album.artist, &album.album),
    };

    let dir = Path::new(&path);

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    Ok(())
}

pub fn file_path(
    album: Arc<String>,
    artist: Arc<String>,
    track: &Track,
    path: Arc<Option<PathBuf>>,
    track_format: Arc<Option<String>>,
) -> Result<PathBuf> {
    let parent = format!("{}/{}", &artist, &album);

    let file_name = match *track_format {
        Some(ref format) => {
            parse_track_template(format, Arc::clone(&album), Arc::clone(&artist), track)
        }
        None => format!("{} - {}", &track.num, &track.name),
    };

    let file = if let Some(ref path) = *path {
        path.join(format!("{}/{}.mp3", parent, file_name))
    } else {
        PathBuf::from(format!("{}/{}.mp3", parent, file_name))
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

pub fn red_cross(msg: &str) -> String {
    format!(
        "{}{}",
        msg,
        Style::new()
            .red()
            .bold()
            .apply_to(if msg.is_empty() { "✘" } else { "" })
    )
}

pub fn green_check(msg: &str) -> String {
    format!(
        "{}{}",
        msg,
        Style::new()
            .green()
            .bold()
            .apply_to(if msg.is_empty() { "✔" } else { "" })
    )
}

pub fn format_container(
    num: String,
    track: String,
    album: String,
    artist: String,
) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("num".to_string(), num);
    vars.insert("track".to_string(), track);
    vars.insert("album".to_string(), album);
    vars.insert("artist".to_string(), artist);
    vars
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
    tags: Arc<Option<String>>,
    album_art_url: Arc<Option<String>>,
    release_date: Option<id3::Timestamp>,
    track: Track,
    path: Arc<Option<PathBuf>>,
    track_format: Arc<Option<String>>,
) -> Result<()> {
    let path = file_path(
        Arc::clone(&album),
        Arc::clone(&artist),
        &track,
        path,
        track_format,
    )?;

    let res = task::block_on(client(&track.url).recv_bytes())?;

    let mut file = task::block_on(async_std::fs::File::create(&path))?;

    let mut res = &res[..];

    task::block_on(async_std::io::copy(&mut res, &mut file))?;

    let album_art = match *album_art_url {
        None => None,
        Some(ref url) => Some(task::block_on(client(url).recv_bytes())?),
    };

    tag_mp3(album, artist, tags, album_art, release_date, &track, &path)?;

    Ok(())
}

pub fn tag_mp3(
    album: Arc<String>,
    artist: Arc<String>,
    tags: Arc<Option<String>>,
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
        tag.add_lyrics(Lyrics {
            lang: "eng".to_string(),
            description: "".to_string(),
            text: String::from(lyrics.as_str()),
        });
    }

    if let Some(ref genre) = *tags {
        tag.set_genre(genre);
    }

    if let Some(album_art) = album_art {
        tag.add_picture(Picture {
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
