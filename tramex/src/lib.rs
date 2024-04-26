#![deny(clippy::all, rust_2018_idioms)]
//#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

mod app;
pub use app::TramexApp;

mod frontend;
pub mod panels;

mod utils;
pub use utils::*;
