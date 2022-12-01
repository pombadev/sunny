//! Sunny is a library for scraping bandcamp.com

/// Track & Album represented as structs
pub mod models;

/// Spider crawls the web, I crawl bandcamp.com
pub mod spider;

/// Miscellaneous small utilities for mostly internal usage
pub mod utils;

/// Client to download single or multiple items
pub mod client;
