pub mod connector;
pub mod errors;
pub mod file_handler;
pub mod functions;

pub mod websocket {
    pub mod layer;
    pub mod log_get;
    pub mod onelog;
    pub mod types;
    pub mod ws_connection;
}

pub mod types {
    pub mod internals;
}

// mod rrc {
//     #![allow(warnings)]
//     include!("./rrc.rs");
// }
