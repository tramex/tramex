use crate::types::file_handler::File;
use crate::types::websocket_types::{Direction, WsConnection};

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
    pub timestamp: String,
    pub msgtype: String,
    pub direction: Direction,
    pub canal: String,
    pub canal_msg: String,
}

pub enum Interface {
    Ws(WsConnection),
    File(File),
}
