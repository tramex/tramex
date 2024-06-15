//! Interface module

use crate::data::Data;
use crate::errors::TramexError;
use crate::interface::interface_file::file_handler::File;
use crate::interface::layer::Layers;
#[cfg(feature = "websocket")]
use crate::interface::websocket::ws_connection::WsConnection;

/// Interface enum
pub enum Interface {
    /// WebSocket connection
    #[cfg(feature = "websocket")]
    Ws(WsConnection),

    /// File
    File(File),
}

/// Interface trait
pub trait InterfaceTrait {
    /// Get more data
    /// # Errors
    /// Return an error if the data is not received correctly
    fn get_more_data(&mut self, _layer_list: Layers, data: &mut Data) -> Result<(), TramexError>;

    /// Try to close the interface
    /// # Errors
    /// Return an error if its fail
    fn close(&mut self) -> Result<(), TramexError>;
}

impl core::fmt::Debug for Interface {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Interface")
            .field(
                "field",
                match self {
                    #[cfg(feature = "websocket")]
                    Interface::Ws(ws) => ws,
                    Interface::File(file) => file,
                },
            )
            .finish()
    }
}
