use std::{error::Error as StdError, fmt, io, mem, result::Result as StdResult};

#[derive(Debug)]
pub enum Error {
    FileExist(String),
    Http(String),
    IO(String),
    Scrape(String),
    Tag(String),
    Thread(String),
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
            Error::FileExist(file) => write!(f, "File {} already exist!", file),
            Error::Http(reason) => write!(f, "{}", reason),
            Error::IO(reason) => write!(f, "{}", reason),
            Error::Scrape(reason) => write!(f, "{}", reason),
            Error::Tag(reason) => write!(f, "{}", reason),
            Error::Thread(reason) => write!(f, "{}", reason),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e.to_string())
    }
}

impl From<id3::Error> for Error {
    fn from(e: id3::Error) -> Self {
        Error::Tag(e.to_string())
    }
}

impl From<surf::Error> for Error {
    fn from(e: surf::Error) -> Self {
        Error::Http(e.to_string())
    }
}

impl From<rayon::ThreadPoolBuildError> for Error {
    fn from(e: rayon::ThreadPoolBuildError) -> Self {
        Error::Thread(e.to_string())
    }
}

pub type Result<T> = StdResult<T, Error>;
