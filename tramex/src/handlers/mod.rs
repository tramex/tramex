//! Module: handlers — UI-only connectors for file and WebSocket

pub mod handler_file;
#[cfg(feature = "websocket")]
pub mod handler_ws;
