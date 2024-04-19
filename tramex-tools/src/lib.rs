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
