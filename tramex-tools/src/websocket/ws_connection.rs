use ewebsock::{WsReceiver, WsSender};

pub struct WsConnection {
    pub ws_sender: Box<WsSender>,
    pub ws_receiver: Box<WsReceiver>,
    pub connecting: bool,
    pub error_str: Option<String>,
}

impl core::fmt::Debug for WsConnection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Interface")
            .field("ws_sender", &"Box<WsSender>")
            .field("ws_receiver", &"Box<WsReceiver>")
            .field("connecting", &self.connecting)
            .field("error_str", &self.error_str)
            .finish()
    }
}
