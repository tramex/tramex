//! WsConnection struct
use core::fmt::{Debug, Formatter};
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};

use crate::interface::interface_types::InterfaceTrait;
use crate::tramex_error;
use crate::{data::Data, errors::TramexError};

use crate::interface::{layer::Layers, log_get::LogGet, types::WebSocketLog};
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

    /// Asking size max
    pub asking_size_max: u64,

    /// Available flag
    pub available: bool,
}

impl WsConnection {
    /// Create a new WsConnection
    pub fn new(ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        Self {
            ws_sender,
            ws_receiver,
            msg_id: 1,
            connecting: true,
            asking_size_max: 1024,
            available: true,
        }
    }

    /// Connect to a WebSocket
    /// # Errors
    /// Return an error as String if the connection failed - see [`ewebsock::connect_with_wakeup`] for more details
    pub fn connect(url: &str, wakeup: impl Fn() + Send + Sync + 'static) -> Result<(WsSender, WsReceiver), String> {
        let options = ewebsock::Options::default();
        ewebsock::connect_with_wakeup(url, options, wakeup)
    }

    /// Try to close the ws
    /// # Errors
    /// Return an error if its fail see [`ewebsock::WsSender::close`] for more details
    pub fn close_impl(&mut self) -> Result<(), TramexError> {
        if let Err(err) = self.ws_sender.close() {
            log::error!("Error closing WebSocket: {}", err);
            return Err(tramex_error!(
                err.to_string(),
                crate::errors::ErrorCode::WebSocketErrorClosing
            ));
        }
        Ok(())
    }
}

impl InterfaceTrait for WsConnection {
    fn get_more_data(&mut self, layer_list: Layers, _data: &mut Data) -> Result<(), TramexError> {
        let msg = LogGet::new(self.msg_id, layer_list, self.asking_size_max);
        log::debug!("Sending message: {:?}", msg);
        match serde_json::to_string(&msg) {
            Ok(msg_stringed) => {
                log::debug!("{}", msg_stringed);
                self.ws_sender.send(WsMessage::Text(msg_stringed));
                self.msg_id += 1;
            }
            Err(err) => {
                log::error!("Error encoding message: {:?}", err);
                return Err(tramex_error!(
                    err.to_string(),
                    crate::errors::ErrorCode::WebSocketErrorEncodingMessage
                ));
            }
        }
        Ok(())
    }

    fn try_recv(&mut self, data: &mut Data) -> Result<(), TramexError> {
        while let Some(event) = self.ws_receiver.try_recv() {
            self.connecting = false;
            match event {
                WsEvent::Message(msg) => {
                    self.available = true;
                    match msg {
                        WsMessage::Text(event_text) => {
                            let decoded: Result<WebSocketLog, serde_json::Error> = serde_json::from_str(&event_text);
                            match decoded {
                                Ok(decoded_data) => {
                                    for one_log in decoded_data.logs {
                                        match one_log.extract_data() {
                                            Ok(trace) => {
                                                data.events.push(trace);
                                            }
                                            Err(err) => {
                                                log::error!("Error while extracting data: {:?}", err);
                                                log::error!("Message: {:?}", event_text);
                                                return Err(err);
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    log::error!("Error decoding message: {:?}", err);
                                    log::error!("Message: {:?}", event_text);
                                    return Err(tramex_error!(
                                        err.to_string(),
                                        crate::errors::ErrorCode::WebSocketErrorDecodingMessage
                                    ));
                                }
                            }
                        }
                        WsMessage::Unknown(str_error) => {
                            log::error!("Unknown message: {:?}", str_error);
                            return Err(tramex_error!(
                                str_error,
                                crate::errors::ErrorCode::WebSocketUnknownMessageReceived
                            ));
                        }
                        WsMessage::Binary(bin) => {
                            log::error!("Unknown binary message: {:?}", bin);
                            return Err(tramex_error!(
                                format!("Unknown binary message: {:?}", bin),
                                crate::errors::ErrorCode::WebSocketUnknownBinaryMessageReceived
                            ));
                        }
                        _ => {
                            log::debug!("Received Ping-Pong")
                        }
                    }
                }
                WsEvent::Opened => {
                    self.available = true;
                    log::debug!("WebSocket opened");
                }
                WsEvent::Closed => {
                    self.available = false;
                    log::debug!("WebSocket closed");
                    return Err(tramex_error!(
                        "WebSocket closed".to_string(),
                        crate::errors::ErrorCode::WebSocketClosed
                    ));
                }
                WsEvent::Error(str_err) => {
                    self.available = false;
                    log::error!("WebSocket error: {:?}", str_err);
                    return Err(tramex_error!(str_err, crate::errors::ErrorCode::WebSocketError));
                }
            }
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), TramexError> {
        self.close_impl()
    }
}

impl Debug for WsConnection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        return formatter
            .debug_struct("Interface")
            .field("ws_sender", &"Box<WsSender>")
            .field("ws_receiver", &"Box<WsReceiver>")
            .field("connecting", &self.connecting)
            .finish();
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
