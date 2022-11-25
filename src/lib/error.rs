use std::{error::Error as StdError, fmt, io, mem, result::Result as StdResult};

macro_rules! impl_from {
    ($ex:ty, $p:tt) => {
        impl From<$ex> for Error {
            fn from(e: $ex) -> Self {
                Self::$p(e.to_string())
            }
        }
    };
}

#[derive(Debug)]
pub enum Error {
    FileExist(String),
    Http(String),
    IO(String),
    Scrape(String),
    Tag(String),
    Thread(String),
    General(String),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        // Thanks: https://stackoverflow.com/a/32554326
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileExist(file) => write!(f, "File {} already exist!", file),
            Self::Http(reason)
            | Self::IO(reason)
            | Self::Scrape(reason)
            | Self::Tag(reason)
            | Self::Thread(reason)
            | Self::General(reason) => write!(f, "{}", reason),
        }
    }
}

impl_from!(io::Error, IO);
impl_from!(id3::Error, Tag);
impl_from!(surf::Error, Http);
impl_from!(rayon::ThreadPoolBuildError, Thread);
impl_from!(&str, General);

pub type Result<T> = StdResult<T, Error>;
