//! Interface module

use crate::errors::TramexError;
use crate::file_handler::File;
#[cfg(feature = "websocket")]
use crate::websocket::ws_connection::WsConnection;

/// Interface enum
pub enum Interface {
    /// WebSocket connection
    #[cfg(feature = "websocket")]
    Ws(WsConnection),

    /// File
    File(File),

    /// None
    None,
}

impl Interface {
    /// close the interface
    /// # Errors
    /// Return an error if the interface is not closed correctly
    pub fn close(&mut self) -> Result<(), TramexError> {
        #[cfg(feature = "websocket")]
        if let Interface::Ws(interface_ws) = self {
            if let Err(err) = interface_ws.ws_sender.close() {
                log::error!("Error closing WebSocket: {}", err);
                return Err(TramexError::new(
                    err.to_string(),
                    crate::errors::ErrorCode::WebSocketErrorClosing,
                ));
            }
        }
        Ok(())
    }

    /// Check if the interface is present
    pub const fn is_some(&self) -> bool {
        match self {
            #[cfg(feature = "websocket")]
            Interface::Ws(_) => true,
            Interface::File(_) => true,
            Interface::None => false,
        }
    }

    /// Check if the interface is None
    pub const fn is_none(&self) -> bool {
        matches!(self, Interface::None)
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
                    #[cfg(feature = "websocket")]
                    Interface::Ws(ws) => ws,
                    Interface::File(file) => file,
                    Interface::None => &"None",
                },
            )
            .finish()
    }
}
