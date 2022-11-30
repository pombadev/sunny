#[derive(Debug, Default, Clone)]
pub struct Track {
    pub num: i32,
    pub name: String,
    pub url: String,
    pub lyrics: Option<String>,
}

impl Track {
    pub fn missing_fields(&self) -> Vec<String> {
        let mut missing = vec![];

        if self.name.is_empty() {
            missing.push("name".to_string());
        }

        if self.num == 0 {
            missing.push("num".to_string());
        }

        if self.url.is_empty() {
            missing.push("url".to_string());
        }

        if self.lyrics.is_none() {
            missing.push("lyrics".to_string());
        }

        missing
    }

    pub fn has_missing_fields(&self) -> bool {
        // name & url are only fields we really need
        self.name.is_empty() || self.url.is_empty()
    }
}

#[derive(Default, Debug)]
pub struct Album {
    pub artist: String,
    pub album: String,
    pub release_date: String,
    pub tracks: Vec<Track>,
    pub tags: Option<String>,
    pub album_art_url: Option<String>,
    pub artist_art_url: Option<String>,
}

impl Album {
    pub fn update(&mut self, other: Self) {
        if !self.album.is_empty() {
            self.album = other.album;
        }

        if !self.artist.is_empty() {
            self.artist = other.artist;
        }

        if !self.release_date.is_empty() {
            self.release_date = other.release_date;
        }

        if !self.tracks.is_empty() {
            self.tracks = other.tracks;
        }

        if let Some(ref tags) = self.tags {
            if tags.is_empty() {
                self.tags = other.tags;
            }
        }

        if self.album_art_url.is_none() {
            self.album_art_url = other.album_art_url;
        }

        if self.artist_art_url.is_none() {
            self.artist_art_url = other.artist_art_url;
        }
    }
}
