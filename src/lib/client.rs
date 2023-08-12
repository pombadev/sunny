use std::env::consts;

use anyhow::Result;
use curl::{easy, Version};

#[path = "./multi_dl.rs"]
mod multi_dl;

pub use multi_dl::Downloader as MultiDownloader;

#[must_use]
pub fn user_agent() -> String {
    format!(
        "{}/{} ({}, {}) curl/{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        consts::OS,
        consts::ARCH,
        Version::get().version()
    )
}

pub fn handle(url: &str) -> Result<easy::Easy> {
    let mut handle = easy::Easy::new();

    handle.useragent(&user_agent())?;
    handle.url(url)?;
    handle.follow_location(true)?;

    Ok(handle)
}

pub fn send(mut handle: easy::Easy) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    let mut transfer = handle.transfer();

    transfer.write_function(|data| {
        buf.extend_from_slice(data);
        Ok(data.len())
    })?;

    transfer.perform()?;

    drop(transfer);

    Ok(buf)
}

pub fn get(url: &str) -> Result<Vec<u8>> {
    let handle = handle(url)?;
    send(handle)
}

pub fn post(url: &str, data: &[u8]) -> Result<Vec<u8>> {
    let mut handle = handle(url)?;

    handle.post(true)?;
    handle.post_fields_copy(data)?;

    send(handle)
}
