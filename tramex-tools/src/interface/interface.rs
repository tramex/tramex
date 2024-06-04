//! Interface module

use crate::data::Data;
use crate::errors::TramexError;
use crate::interface::file_handler::file_handler::File;
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
    /// Receive data
    /// # Errors
    /// Return an error if the data is not received correctly
    fn get_more_data(
        &mut self,
        _layer_list: Layers,
        max_size: u64,
        data: &mut Data,
        available: &mut bool,
    ) -> Result<(), TramexError>;

    /// Try to receive data
    /// # Errors
    /// Return an error if the data is not received correctly
    fn try_recv(&mut self, data: &mut Data, available: &mut bool) -> Result<(), TramexError>;

    /// Try to close the interface
    /// # Errors
    /// Return an error if its fail
    fn close(&mut self) -> Result<(), TramexError>;
}

impl InterfaceTrait for Interface {
    /// Receive data
    /// # Errors
    /// Return an error if the data is not received correctly
    fn get_more_data(
        &mut self,
        _layer_list: Layers,
        max_size: u64,
        data: &mut Data,
        available: &mut bool,
    ) -> Result<(), TramexError> {
        match self {
            #[cfg(feature = "websocket")]
            Interface::Ws(ws) => ws.get_more_data(_layer_list, max_size, data, available),
            Interface::File(file) => file.get_more_data(_layer_list, max_size, data, available),
        }
    }

    /// Try to receive data
    /// # Errors
    /// Return an error if the data is not received correctly
    fn try_recv(&mut self, data: &mut Data, available: &mut bool) -> Result<(), TramexError> {
        match self {
            #[cfg(feature = "websocket")]
            Interface::Ws(ws) => ws.try_recv(data, available),
            Interface::File(file) => file.try_recv(data, available),
        }
    }

    fn close(&mut self) -> Result<(), TramexError> {
        match self {
            #[cfg(feature = "websocket")]
            Interface::Ws(ws) => ws.close(),
            Interface::File(file) => file.close(),
        }
    }
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
