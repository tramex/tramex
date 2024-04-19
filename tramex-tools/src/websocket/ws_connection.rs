use ewebsock::{WsReceiver, WsSender};

pub struct WsConnection {
    pub ws_sender: Box<WsSender>,
    pub ws_receiver: Box<WsReceiver>,
    pub msg_id: u64,
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
