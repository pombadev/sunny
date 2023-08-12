use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use console::style;
use curl::easy::List;
use html_escape::decode_html_entities;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use scraper::{Html, Selector};

use crate::{
    client,
    models::{Album, Track},
};

fn find_track_by_name(dom: &Html, track_name: &gjson::Value) -> Option<Track> {
    let album = scrape_by_data_tralbum(dom);

    album.tracks.iter().find_map(|inner_track| {
        if inner_track.name == track_name.str() {
            Some(inner_track.clone())
        } else {
            None
        }
    })
}

/// Parse data from the node: `document.querySelector('script[type="application/ld+json"]')`
fn scrape_by_application_ld_json(dom: &Html) -> Option<Album> {
    let selector = Selector::parse("script[type='application/ld+json']").unwrap();
    let element = dom.select(&selector).next().unwrap();

    let json = element.inner_html();
    let json = json.as_str();

    if !gjson::valid(json) {
        return None;
    }

    let mut album = Album::default();

    let item = gjson::get(json, "@this");

    album.album = item.get("name").to_string();

    let tags = item
        .get("keywords")
        .array()
        .iter()
        .filter_map(|tag| {
            let tag = tag.str().trim();

            if tag.is_empty() {
                None
            } else {
                Some(tag)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    album.tags = Some(tags);
    album.release_date = item.get("datePublished").to_string();
    album.album_art_url = Some(item.get("image").to_string());
    album.artist = item.get("byArtist.name").to_string();
    album.artist_art_url = Some(item.get("byArtist.image").to_string());

    let tracks = item.get("track.itemListElement");

    const FILE_PATH: &str = "additionalProperty.#(name=file_mp3-128).value";

    // case when current url is just a track
    album.tracks = if tracks.array().is_empty() {
        let mut url = decode_html_entities(&item.get(FILE_PATH).to_string()).to_string();
        let track_name = item.get("name");

        if url.is_empty() {
            if let Some(track_url) = find_track_by_name(dom, &track_name) {
                url = track_url.url;
            } else {
                // no url is found for the track's file
                // eprintln!("No downloadable url found for '{}', skipping.", &track_name);
                return None;
            }
        }

        vec![Track {
            num: 1,
            name: track_name.to_string(),
            url,
            lyrics: None,
            album: album.clone(),
        }]
    } else {
        // case when current url is an album
        tracks
            .array()
            .iter()
            .filter_map(|track| {
                let mut url = track.get(&("item".to_owned() + FILE_PATH)).to_string();

                if url.is_empty() {
                    let track_name = track.get("item.name");

                    if let Some(track_url) = find_track_by_name(dom, &track_name) {
                        url = track_url.url;
                    } else {
                        // no url is found for the track's file
                        // eprintln!("No downloadable url found for '{}', skipping.", track_name);
                        return None;
                    }
                }

                Some(Track {
                    num: track.get("position").i32(),
                    name: decode_html_entities(&track.get("item.name").to_string()).into(),
                    url: decode_html_entities(&url).to_string(),
                    lyrics: Some(track.get("item.recordingOf.lyrics.text").to_string()),
                    album: album.clone(),
                })
            })
            .collect()
    };

    Some(album)
}

/// Parse data from the node: `document.querySelector('script[data-tralbum]')`
fn scrape_by_data_tralbum(dom: &Html) -> Album {
    let selector = Selector::parse("script[data-tralbum]").unwrap();
    let element = dom.select(&selector).next().unwrap();

    let mut album = Album::default();

    for (name, val) in &element.value().attrs {
        let data = gjson::get(val.trim(), "@this");

        if &name.local == "data-embed" {
            album.artist = data.get("artist").to_string();
            album.album = data.get("album_title").to_string();
        }

        if &name.local == "data-tralbum" {
            if album.album.is_empty() {
                album.album = data.get("current.title").to_string();
            }

            album.release_date = data.get("album_release_date").to_string();
            album.tracks = data
                .get("trackinfo")
                .array()
                .iter()
                .enumerate()
                .map(|(index, item)| Track {
                    num: (index + 1) as i32,
                    name: item.get("title").to_string(),
                    url: item.get("file.mp3-128").to_string(),
                    lyrics: None,
                    album: album.clone(),
                })
                .collect();
        }
    }

    album
}

/// Scrape album links from `/music` or `/releases` page.
fn get_all_album_links(dom: &Html) -> Vec<String> {
    // js equivalent: document.querySelectorAll("#music-grid > li > a")
    let albums_selector = Selector::parse("#music-grid > li > a").unwrap();

    // get artist base url, js equivalent: document.querySelector(`meta[property="og:url"]`)
    let base_url_selector = Selector::parse("meta[property='og:url']").unwrap();

    let url_selector = dom.select(&base_url_selector).next().unwrap();

    if let Some(base_url) = url_selector.value().attr("content") {
        dom.select(&albums_selector)
            .filter_map(|el| el.value().attr("href"))
            .map(|album| base_url.to_owned() + album)
            .collect::<Vec<_>>()
    } else {
        vec![]
    }
}

/// Facade for `scrape_by_*` methods.
/// Calls `scrape_by_application_ld_json` or `scrape_by_data_tralbum` internal methods if first fails.
fn get_album(dom: &Html) -> Option<Album> {
    scrape_by_application_ld_json(dom)
}

/// Get [`Html`] of a page.
fn fetch_html(url: &str) -> Result<Html> {
    let body = client::get(url)?;
    let body = String::from_utf8(body)?;

    Ok(Html::parse_document(body.as_ref()))
}

/// Fetch albums
pub fn fetch_albums(url: &str) -> Result<Vec<Album>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner().template("{spinner} {prefix} {msg} ({elapsed})")?,
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_prefix("Fetching artist's info");

    let html = fetch_html(url).map_err(|err| {
        pb.finish_with_message(style("✘").bold().red().to_string());
        anyhow!("{err}")
    })?;

    let is_album = html.select(&Selector::parse("#trackInfo").unwrap()).count() > 0;

    if is_album {
        let album = get_album(&html);

        pb.finish_with_message(style("✔").bold().green().to_string());

        return Ok(album.into_iter().collect());
    }

    #[rustfmt::skip]
    let is_discography = html
        .select(&Selector::parse("#music-grid").unwrap())
        .count() > 0;

    if is_discography {
        let albums = get_all_album_links(&html)
            .par_iter()
            .filter_map(|url| fetch_html(url).map_or(None, |dom| get_album(&dom)))
            .collect::<Vec<_>>();

        pb.finish_with_message(style("✔").bold().green().to_string());

        return Ok(albums);
    }

    // this should never reach, however if it does throw an error.
    bail!("Invalid page.")
}

pub fn search(query: &str, query_type: &str) -> Result<()> {
    let data = format!(
        r#"{{"search_text":"{query}","search_filter":"{query_type}","full_page":true,"fan_id":null}}"#
    );

    let mut handle =
        client::handle("https://bandcamp.com/api/bcsearch_public_api/1/autocomplete_elastic")?;

    let mut headers = List::new();

    headers.append("Content-Type: application/json")?;
    handle.http_headers(headers)?;
    handle.post(true)?;
    handle.post_fields_copy(data.as_bytes())?;

    let response = client::send(handle)?;
    let json = unsafe { gjson::get_bytes(response.as_slice(), "auto.results") };
    let results = json.array();

    if results.is_empty() {
        eprintln!("No matching results\nTry a different `--type` or a new search keyword.");
        return Ok(());
    }

    use term_table::{
        row::Row,
        table_cell::{Alignment, TableCell},
        Table, TableStyle,
    };

    let mut table = Table::new();

    table.style = TableStyle::blank();

    table.separate_rows = false;

    table.add_row(Row::new(vec![
        TableCell::new_with_alignment(style("Name").bold().underlined(), 1, Alignment::Left),
        TableCell::new_with_alignment(style("Location").bold().underlined(), 1, Alignment::Left),
        TableCell::new_with_alignment(style("Genre").bold().underlined(), 1, Alignment::Left),
        TableCell::new_with_alignment(style("Url").bold().underlined(), 1, Alignment::Left),
    ]));

    let mut items = json.array();

    items.sort_by_key(|item| item.get("name").str().to_lowercase());

    for item in items {
        table.add_row(Row::new(vec![
            TableCell::new_with_alignment(item.get("name").to_string(), 1, Alignment::Left),
            TableCell::new_with_alignment(item.get("location").to_string(), 1, Alignment::Left),
            TableCell::new_with_alignment(item.get("genre_name").to_string(), 1, Alignment::Left),
            TableCell::new_with_alignment(
                item.get("item_url_root").to_string(),
                1,
                Alignment::Left,
            ),
        ]));
    }

    print!("{}", table.render().trim());

    Ok(())
}
