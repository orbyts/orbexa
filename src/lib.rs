//! Core library for Orbexa.
//!
//! Orbexa applies Codexa-generated Notion artifacts to managed Notion
//! pages, databases, and data sources.

pub mod artifact;
pub mod config;
pub mod lock;
pub mod notion;
pub mod plan;
pub mod registry;
pub mod render;
pub mod state;

/// Current Orbexa package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
