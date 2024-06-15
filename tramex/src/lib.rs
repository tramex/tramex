//! Tramex is a 4G frame analyzer and visualizer
#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo
)]
#![warn(clippy::multiple_crate_versions)]

mod app;
pub use app::TramexApp;

mod frontend;
pub mod handlers;
pub mod panels;

mod utils;
pub use utils::*;
