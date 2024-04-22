#![deny(clippy::all, rust_2018_idioms)]
//#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

pub mod connector;
pub mod data;
pub mod errors;
pub mod file_handler;
pub mod functions;
pub mod interface;

pub mod websocket {
    pub mod layer;
    pub mod log_get;
    pub mod onelog;
    pub mod types;
    pub mod ws_connection;
}

// mod rrc {
//     #![allow(warnings)]
//     include!("./rrc.rs");
// }
