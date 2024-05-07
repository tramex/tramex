//! WsConnection struct
use ewebsock::{WsReceiver, WsSender};

/// WsConnection struct
pub struct WsConnection {
    /// WebSocket sender
    pub ws_sender: WsSender,

    /// WebSocket receiver
    pub ws_receiver: WsReceiver,

    /// Message ID
    pub msg_id: u64,

    /// Connecting flag
    pub connecting: bool,
}

impl core::fmt::Debug for WsConnection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Interface")
            .field("ws_sender", &"Box<WsSender>")
            .field("ws_receiver", &"Box<WsReceiver>")
            .field("connecting", &self.connecting)
            .finish()
    }
}

impl Drop for WsConnection {
    fn drop(&mut self) {
        log::debug!("Cleaning WsConnection");
        if let Err(err) = self.ws_sender.close() {
            log::error!("Error closing WebSocket: {}", err);
        }
    }
}
