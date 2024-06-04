//! WsConnection struct
use core::fmt::{Debug, Formatter};
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};

use crate::{
    data::{Data, MessageType, Trace},
    errors::TramexError,
};

use crate::interface::{interface_types::InterfaceTrait, layer::Layers, log_get::LogGet, types::WebSocketLog};
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

impl WsConnection {
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
            return Err(TramexError::new(
                err.to_string(),
                crate::errors::ErrorCode::WebSocketErrorClosing,
            ));
        }
        Ok(())
    }
}

impl InterfaceTrait for WsConnection {
    fn get_more_data(
        &mut self,
        layer_list: Layers,
        max_size: u64,
        _data: &mut Data,
        _available: &mut bool,
    ) -> Result<(), TramexError> {
        let msg = LogGet::new(self.msg_id, layer_list, max_size);
        match serde_json::to_string(&msg) {
            Ok(msg_stringed) => {
                log::debug!("{}", msg_stringed);
                self.ws_sender.send(WsMessage::Text(msg_stringed));
                self.msg_id += 1;
            }
            Err(err) => {
                log::error!("Error encoding message: {:?}", err);
                return Err(TramexError::new(
                    err.to_string(),
                    crate::errors::ErrorCode::WebSocketErrorEncodingMessage,
                ));
            }
        }
        Ok(())
    }

    fn try_recv(&mut self, data: &mut Data, available: &mut bool) -> Result<(), TramexError> {
        while let Some(event) = self.ws_receiver.try_recv() {
            self.connecting = false;
            match event {
                WsEvent::Message(msg) => {
                    *available = true;
                    match msg {
                        WsMessage::Text(event_text) => {
                            let decoded: Result<WebSocketLog, serde_json::Error> = serde_json::from_str(&event_text);
                            match decoded {
                                Ok(decoded_data) => {
                                    for one_log in decoded_data.logs {
                                        let canal_msg = one_log.extract_canal_msg().unwrap_or("".to_owned());
                                        let hexa = one_log.extract_hexe();
                                        let msg_type = MessageType {
                                            timestamp: one_log.timestamp.to_owned(),
                                            layer: one_log.layer,
                                            direction: one_log.dir.unwrap_or_default(),
                                            canal: one_log.channel.unwrap_or_default(),
                                            canal_msg,
                                        };
                                        let trace = Trace {
                                            trace_type: msg_type,
                                            hexa: hexa.unwrap_or_default(),
                                            text: Some(one_log.data),
                                        };
                                        data.events.push(trace);
                                    }
                                }
                                Err(err) => {
                                    log::error!("Error decoding message: {:?}", err);
                                    log::error!("Message: {:?}", event_text);
                                    return Err(TramexError::new(
                                        err.to_string(),
                                        crate::errors::ErrorCode::WebSocketErrorDecodingMessage,
                                    ));
                                }
                            }
                        }
                        WsMessage::Unknown(str_error) => {
                            log::error!("Unknown message: {:?}", str_error);
                            return Err(TramexError::new(
                                str_error,
                                crate::errors::ErrorCode::WebSocketUnknownMessageReceived,
                            ));
                        }
                        WsMessage::Binary(bin) => {
                            log::error!("Unknown binary message: {:?}", bin);
                            return Err(TramexError::new(
                                format!("Unknown binary message: {:?}", bin),
                                crate::errors::ErrorCode::WebSocketUnknownBinaryMessageReceived,
                            ));
                        }
                        _ => {
                            log::debug!("Received Ping-Pong")
                        }
                    }
                }
                WsEvent::Opened => {
                    *available = true;
                    log::debug!("WebSocket opened");
                }
                WsEvent::Closed => {
                    *available = false;
                    log::debug!("WebSocket closed");
                    return Err(TramexError::new(
                        "WebSocket closed".to_string(),
                        crate::errors::ErrorCode::WebSocketClosed,
                    ));
                }
                WsEvent::Error(str_err) => {
                    *available = false;
                    log::error!("WebSocket error: {:?}", str_err);
                    return Err(TramexError::new(str_err, crate::errors::ErrorCode::WebSocketError));
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
