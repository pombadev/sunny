use anyhow::{bail, Result};
use id3::{
    frame::{Lyrics, Picture, PictureType},
    Tag, TagLike, Timestamp, Version,
};
use strfmt::strfmt;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use super::models::{Album, Track};

pub fn prepare_directory(path: Option<&PathBuf>, album: &Album) -> Result<PathBuf> {
    let path = match path {
        Some(path) => Path::new(path).join(&album.artist),
        None => PathBuf::from(&album.artist),
    }
    .join(&album.album);

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    Ok(path)
}

#[must_use]
pub fn make_path(track: &Track, root: &Path, track_format: &String) -> PathBuf {
    let file_name = if track_format.is_empty() {
        format!("{} - {}", &track.num, &track.name)
    } else {
        parse_track_template(track_format, track)
    };

    root.join(file_name).with_extension("mp3")
}

pub fn track_path(track: &Track, root: &Path, track_format: &String) -> Result<PathBuf> {
    let file_name = if track_format.is_empty() {
        format!("{} - {}", &track.num, &track.name)
    } else {
        parse_track_template(track_format, track)
    };

    let file = root.join(file_name).with_extension("mp3");

    if file.exists() {
        bail!("{} already exists", file.display())
    }

    Ok(file)
}

#[must_use]
pub fn timestamp(date_string: &str) -> Option<Timestamp> {
    use chrono::{DateTime, Datelike, Timelike};

    // if String looks like this `28 Sep 2014 04:19:31 GMT`
    match DateTime::parse_from_str(date_string, "%d %b %Y %T %Z") {
        Ok(date) => Some(Timestamp {
            year: date.year(),
            month: Some(date.month() as u8),
            day: Some(date.day() as u8),
            hour: Some(date.hour() as u8),
            minute: Some(date.minute() as u8),
            second: Some(date.second() as u8),
        }),
        Err(_) => {
            // if String looks like this `released September 28, 2014`
            if date_string.starts_with("released ") {
                let mut date_string = date_string.to_owned();
                date_string.push_str(" 01:01:01");

                match DateTime::parse_from_str(&date_string, "released %B %d, %Y %T") {
                    Ok(date) => Some(Timestamp {
                        year: date.year(),
                        month: Some(date.month() as u8),
                        day: Some(date.day() as u8),
                        hour: Some(date.hour() as u8),
                        minute: Some(date.minute() as u8),
                        second: Some(date.second() as u8),
                    }),
                    Err(_) => None,
                }
            } else {
                None
            }
        }
    }
}

#[must_use]
pub fn format_container(
    num: &str,
    track: &str,
    album: &str,
    artist: &str,
) -> HashMap<String, String> {
    HashMap::from([
        ("num".to_string(), num.to_owned()),
        ("track".to_string(), track.to_owned()),
        ("album".to_string(), album.to_owned()),
        ("artist".to_string(), artist.to_owned()),
    ])
}

#[must_use]
pub fn parse_track_template(format: &str, track: &Track) -> String {
    let Album { album, artist, .. } = &track.album;
    let Track { ref num, name, .. } = track;

    let vars = format_container(&num.to_string(), name, album, artist);

    strfmt(format, &vars).expect("failed to format keys")
}

pub fn tag_mp3(
    album_art: Option<Vec<u8>>,
    release_date: Option<id3::Timestamp>,
    track: &Track,
    path: &Path,
) -> Result<()> {
    let album = &track.album;
    let mut tag = Tag::new();

    tag.set_title(&*track.name);
    tag.set_track(track.num as u32);
    tag.set_album(&album.album);
    tag.set_artist(&album.artist);
    tag.set_album_artist(&album.artist);

    if let Some(ref lyrics) = track.lyrics {
        tag.add_frame(Lyrics {
            lang: "eng".to_string(),
            description: String::with_capacity(0),
            text: String::from(lyrics.as_str()),
        });
    }

    if let Some(tags) = &album.tags {
        tag.set_genre(tags.to_string());
    }

    if let Some(album_art) = album_art {
        tag.add_frame(Picture {
            mime_type: "image/jpeg".to_string(),
            picture_type: PictureType::CoverFront,
            description: String::with_capacity(0),
            data: album_art,
        });
    }

    if let Some(ts) = release_date {
        tag.set_date_recorded(ts);
    }

    tag.write_to_path(path, Version::Id3v24)?;

    Ok(())
}

pub fn print_as_tree(albums: &[Album]) {
    if albums.is_empty() {
        println!("Noting to print");
    } else {
        let artist = &albums[0].artist;

        println!("{artist}");

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
