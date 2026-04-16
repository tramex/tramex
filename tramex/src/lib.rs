//! Tramex is a 4G frame analyzer and visualizer
#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo
)]
#![allow(clippy::multiple_crate_versions)]

mod app;
pub use app::TramexApp;

pub mod event_system;
mod frontend;
pub mod handlers;
pub mod panels;

mod utils;
pub use utils::*;

pub mod theme;
pub use theme::{ArrowColors, ChannelColors, ThemeColors};

#[cfg(feature = "ai")]
pub mod ai_settings;
