//! Core library for Orbexa.
//!
//! Orbexa applies Codexa-generated Notion artifacts to managed Notion
//! pages, databases, and data sources.

pub mod config;
pub mod notion;
pub mod plan;
pub mod registry;
pub mod state;

/// Current Orbexa package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
