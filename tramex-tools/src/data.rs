use crate::websocket::{layer::Layer, types::Direction};
use core::fmt::Debug;

#[derive(Debug)]
pub struct Data {
    pub events: Vec<Trace>,
    pub current_index: usize,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            events: Default::default(),
            current_index: 0,
        }
    }
}

#[derive(Debug)]
pub struct Trace {
    pub trace_type: MessageType,
    pub hexa: Vec<u8>,
}
#[derive(Debug)]
pub struct MessageType {
    pub timestamp: u64,
    pub layer: Layer,
    pub direction: Direction,
    pub canal: String,
    pub canal_msg: String,
}
