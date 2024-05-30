//! This module contains the data structures used to store the data of the application.
use crate::websocket::{layer::Layer, types::Direction};
use core::fmt::Debug;

#[derive(Debug)]
/// Data structure to store Trace of the application.
pub struct Data {
    /// Vector of Trace.
    pub events: Vec<Trace>,
    /// Current index of the vector.
    pub current_index: usize,
}

impl Default for Data {
    fn default() -> Self {
        let default_data_size = 2048;
        Self {
            events: Vec::with_capacity(default_data_size),
            current_index: 0,
        }
    }
}

#[derive(Debug)]
/// Data structure to store Trace of the application.
pub struct Trace {
    /// Message type.
    pub trace_type: MessageType,

    /// Hexadecimal representation of the message.
    pub hexa: Vec<u8>,

    #[cfg(feature = "debug-trame")]
    /// Text representation of the message from the API
    pub text: Vec<String>,
}
#[derive(Debug)]
/// Data structure to store the message type (from the amarisoft API)
pub struct MessageType {
    /// Timestamp of the message.
    pub timestamp: u64,

    /// Layer of the message.
    pub layer: Layer,

    /// Direction of the message.
    pub direction: Direction,

    /// canal of the message.
    pub canal: String,

    /// Message of the canal.
    pub canal_msg: String,
}
