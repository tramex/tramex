use crate::websocket::{layer::Layer, types::Direction};
use core::fmt::Debug;

#[derive(Debug)]
pub struct Data {
    pub events: Vec<Trace>,
    pub current_index: usize,
}

impl Data {
    pub fn get_current_trace(&self) -> Option<&Trace>{
        return self.events.get(self.current_index);
    }
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
