//! Sunny is a library for scraping bandcamp.com

/// Track & Album represented as structs
pub mod models;

/// Spider crawls the web, I crawl bandcamp.com
pub mod spider;

/// Miscellaneous small utilities for mostly internal usage
pub mod utils;

/// Download multiple tracks simultaneously
pub mod multi_dl;
