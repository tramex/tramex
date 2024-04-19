use crate::errors::TramexError;
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

impl Interface {
    pub fn close(&mut self) -> Result<(), TramexError> {
        match self {
            Interface::Ws(interface_ws) => {
                if let Err(err) = interface_ws.ws_sender.close() {
                    log::error!("Error closing WebSocket: {}", err);
                    return Err(TramexError::new(err.to_string(), 7));
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub const fn is_some(&self) -> bool {
        match self {
            Interface::Ws(_) => true,
            Interface::File(_) => true,
            Interface::None => false,
        }
    }
    pub const fn is_none(&self) -> bool {
        match self {
            Interface::None => true,
            _ => false,
        }
    }
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
