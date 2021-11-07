use async_std::task;
use html_escape::decode_html_entities;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use scraper::{Html, Selector};

use crate::{
    error::{Error, Result},
    models::{Album, Track},
    utils::{client, green_check, red_cross},
};

/// Parse data from the node: `document.querySelector('script[type="application/ld+json"]')`
fn scrape_by_application_ld_json(dom: &Html) -> Option<Album> {
    let album = Selector::parse("script[type='application/ld+json']")
        .into_iter()
        .fold(None, |_, selector| {
            dom.select(&selector).take(1).fold(None, |_, element| {
                let mut album = Album::default();

                let json = element.inner_html();
                let json = json.as_str();

                if !gjson::valid(json) {
                    return None;
                }

                let item = gjson::get(json, "@this");

                album.album = item.get("name").to_string();
                let tags = item
                    .get("keywords")
                    .array()
                    .iter()
                    .map(|tag| String::from(tag.str().trim()))
                    .collect::<Vec<String>>()
                    .join(", ");

                album.tags = Some(tags);
                album.release_date = item.get("datePublished").to_string();
                album.album_art_url = Some(item.get("image").to_string());
                album.artist = item.get("byArtist.name").to_string();
                album.artist_art_url = Some(item.get("byArtist.image").to_string());

                let tracks = item.get("track.itemListElement");

                // case when current url is an album
                album.tracks = if !tracks.array().is_empty() {
                    tracks
                        .array()
                        .iter()
                        .map(|track| Track {
                            num: track.get("position").i32(),
                            name: decode_html_entities(&track.get("item.name").to_string())
                                .to_string(),
                            url: decode_html_entities(
                                &track
                                    .get("item.additionalProperty.#(name=file_mp3-128).value")
                                    .to_string(),
                            )
                            .to_string(),
                            lyrics: Some(track.get("item.recordingOf.lyrics.text").to_string()),
                        })
                        .collect()
                } else {
                    // case when current url is just a track

                    let url = decode_html_entities(
                        &item
                            .get("additionalProperty.#(name=file_mp3-128).value")
                            .to_string(),
                    )
                    .to_string();

                    vec![Track {
                        num: 1,
                        name: item.get("name").to_string(),
                        url,
                        lyrics: None,
                    }]
                };

                Some(album)
            })
        });

    // dbg!(&album);

    album
}

/// Parse data from the node: `document.querySelector('script[data-tralbum]')`
fn scrape_by_data_tralbum(dom: &Html) -> Option<Album> {
    let album = Selector::parse("script[data-tralbum]")
        .into_iter()
        .fold(None, |_, selector| {
            dom.select(&selector).take(1).fold(None, |_, element| {
                let mut album = Album::default();

                for (name, val) in element.value().attrs.iter() {
                    let data = gjson::get(val.trim(), "@this");

                    if &name.local == "data-embed" {
                        album.artist = data.get("artist").to_string();
                        album.album = data.get("album_title").to_string();
                    }

                    if &name.local == "data-tralbum" {
                        if album.album.is_empty() {
                            album.album = data.get("current.title").to_string();
                        }

                        // let data_tralbum = gjson::get(val.trim(), "@this");

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
                            })
                            .collect();
                    }
                }

                Some(album)
            })
        });

    // dbg!(&album);

    album
}

/// Scrape album links from `/music` or `/releases` page.
fn get_all_album_links(dom: &Html) -> Vec<String> {
    // js equivalent: document.querySelectorAll("#music-grid > li > a")
    let albums_link = Selector::parse("#music-grid > li > a")
        .into_iter()
        .take(1)
        .fold(Vec::with_capacity(0), |_, albums_selector| {
            // get artist base url, js equivalent: document.querySelector(`meta[property="og:url"]`).content
            let base_url = Selector::parse("meta[property='og:url']")
                .into_iter()
                .take(1)
                .fold(String::with_capacity(0), |_, url_selector| {
                    dom.select(&url_selector)
                        .take(1)
                        .filter_map(|node|
                            // extract node's `.content`
                            node.value().attr("content"))
                        .fold(String::new(), |mut acc, curr| {
                            acc.push_str(curr);
                            acc
                        })
                });

            let albums = dom
                .select(&albums_selector)
                .filter_map(|el| el.value().attr("href"))
                .map(|album| format!("{}{}", base_url, album))
                .collect::<Vec<_>>();

            albums
        });

    // dbg!(&albums_link);

    albums_link
}

/// Facade for `scrape_by_*` methods.
/// Calls `scrape_by_application_ld_json` or `scrape_by_data_tralbum` internal methods if first fails.
fn get_album(dom: &Html) -> Option<Album> {
    scrape_by_application_ld_json(dom).map_or_else(|| scrape_by_data_tralbum(dom), Some)

    // match strategy_one(&doc) {
    //     Some(mut first) => {
    //         if first.required_fields_missing() {
    //             match strategy_two(&doc) {
    //                 Some(second) => {
    //                     first.update(second);

    //                     if first.required_fields_missing() {
    //                         None
    //                     } else {
    //                         Some(first)
    //                     }
    //                 }
    //                 None => None,
    //             }
    //         } else {
    //             Some(first)
    //         }
    //     }
    //     None => match strategy_two(&doc) {
    //         Some(second) => {
    //             if second.required_fields_missing() {
    //                 None
    //             } else {
    //                 Some(second)
    //             }
    //         }
    //         None => None,
    //     },
    // }
}

/// Get [`Html`] of a page.
fn fetch_html(url: &str, pb: Option<&ProgressBar>) -> Result<Html> {
    let mut res = task::block_on(client(url)).map_err(|e| {
        if let Some(pb) = pb {
            pb.finish_with_message(red_cross(""));
        }
        Error::Http(e.to_string())
    })?;

    let status = &res.status();

    if *status == surf::StatusCode::NotFound {
        if let Some(pb) = pb {
            pb.finish_with_message(red_cross(""));
        }

        return Err(Error::Http(status.canonical_reason().to_string()));
    }

    let body = task::block_on(res.body_string())
        .map(|res| {
            if let Some(pb) = pb {
                pb.finish_with_message(green_check(""));
            }
            res
        })
        .map_err(|e| {
            if let Some(pb) = pb {
                pb.finish_with_message(red_cross(""));
            }
            Error::Http(e.to_string())
        })?;

    Ok(Html::parse_document(body.as_ref()))
}

/// Fetch albums
pub fn fetch_albums(url: &str) -> Result<Vec<Album>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {prefix} {msg} ({elapsed})"));
    pb.enable_steady_tick(100);
    pb.set_prefix("Fetching artist's info");

    let html = fetch_html(url, Some(&pb))?;

    #[rustfmt::skip]
    let is_album = html
        .select(&Selector::parse("#trackInfo")
        .map_err(|err| Error::Scrape(format!("{:?}", &err)))?)
        .count() > 0;

    if is_album {
        let album = get_album(&html);

        pb.finish_with_message(green_check(""));

        return Ok(album.into_iter().collect());
    }

    #[rustfmt::skip]
    let is_discography = html
        .select(&Selector::parse("#music-grid")
        .map_err(|err| Error::Scrape(format!("{:?}", &err)))?)
        .count() > 0;

    if is_discography {
        let albums = get_all_album_links(&html)
            .par_iter()
            .filter_map(|url| {
                if let Ok(dom) = fetch_html(url, None) {
                    get_album(&dom)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        pb.finish_with_message(green_check(""));

        return Ok(albums);
    }

    // this should never reach, however if it does throw an error.
    Err(Error::Http("Invalid page.".to_owned()))
}
