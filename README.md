<h1 align="center">Welcome to sunny 🌞</h1>
<p align="center">
  <a href="https://crates.io/crates/sunny" target="_blank">
    <img alt="Version" src="https://img.shields.io/crates/v/sunny.svg">
  </a>
  <a href="https://docs.rs/sunny" target="_blank">
    <img alt="docs" src="https://docs.rs/sunny/badge.svg" />
  </a>
  <img alt="License: MIT" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue" />
</p>

> Tool to download freely available music from bandcamp.
>
> Automatically organize files to folder, ID3 tags (including album art).

### Demo

[![Demo](./assets/demo.svg)](./assets/demo.svg)


### Motivation

Sunny is hugely inspired by [SoundScrape](https://github.com/Miserlou/SoundScrape), the main motivation  for writing this was speed and customizability.

- SoundScrape downloads sequentially whereas Sunny does parallelly, giving a huge boost of speed.
- Track format can be customized.

### Format
By default files are saved in this structure in current directory if `--path` option is not passed.

```
Artist
  ├── Album
  │   ├── 01 - Track.mp3
  │   ├── 02 - Track.mp3
  │   ├── 03 - Track.mp3
  │   ├── 04 - Track.mp3
```

## Install

```sh
cargo install sunny
```

## Usage

```sh
# whole discography of an artist
sunny -u https://65daysofstatic.bandcamp.com/music

# whole discography of an artist
sunny -u 65daysofstatic

# single album
sunny -u https://clevergirl.bandcamp.com/album/no-drum-and-bass-in-the-jazz-room

# single track
sunny -u https://65daysofstatic.bandcamp.com/track/twenty-four-twelve-twenty
```

<!-- ## Run tests

```sh
cargo test
``` -->

## Contributing

Contributions, issues and feature requests are welcome!
