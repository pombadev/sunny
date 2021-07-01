//! Sunny is a library for scraping bandcamp.com

/// Errors produced by sunny
pub mod error;

/// Track & Album represented as structs
pub mod models;

/// Spider crawls the web, I crawl bandcamp.com
pub mod spider;

/// Miscellaneous small utilities for mostly internal usage
pub mod utils;
