use crate::file_handler::File;
use crate::websocket::{layer::Layer, types::Direction, ws_connection::WsConnection};
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

pub enum Interface {
    Ws(WsConnection),
    File(File),
    None,
}

impl Default for Interface {
    fn default() -> Self {
        Self::None
    }
}

impl core::fmt::Debug for Interface {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Interface")
            .field(
                "field",
                match self {
                    Interface::Ws(ws) => ws,
                    Interface::File(file) => file,
                    Interface::None => &"None",
                },
            )
            .finish()
    }
}
