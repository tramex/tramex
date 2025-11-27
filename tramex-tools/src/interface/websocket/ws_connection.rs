//! WsConnection struct
use core::fmt::{Debug, Formatter};
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use std::vec;

use crate::interface::interface_types::InterfaceTrait;
use crate::interface::types::BaseMessage;
use crate::tramex_error;
use crate::{data::Data, errors::TramexError};

use crate::interface::{layer::Layers, layer::Layer, log_get::LogGet, types::WebSocketLog};
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

    /// Name of the receiver
    pub name: String,

    /// Auto-loading flag - when true, automatically requests more data
    pub auto_loading: bool,

    /// Waiting for response flag - true when a request has been sent and we're waiting for the response
    pub waiting_for_response: bool,
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
            name: "".to_string(),
            auto_loading: true, // Auto-loading enabled by default
            waiting_for_response: false,
        }
    }

    /// Connect to a WebSocket
    /// # Errors
    /// Return an error as String if the connection failed - see [`ewebsock::connect_with_wakeup`] for more details
    pub fn connect(url: &str, wakeup: impl Fn() + Send + Sync + 'static) -> Result<(WsSender, WsReceiver), String> {
        let options = ewebsock::Options {
            #[cfg(not(target_arch = "wasm32"))]
            additional_headers: vec![("Origin".to_string(), "tramex".to_string())],
            ..Default::default()
        };
        ewebsock::connect_with_wakeup(url, options, wakeup)
    }

    /// Try to close the ws
    /// # Errors
    /// Return an error if its fail see [`ewebsock::WsSender::close`] for more details
    pub fn close_impl(&mut self) -> Result<(), TramexError> {
        self.ws_sender.close();
        Ok(())
    }
}

impl InterfaceTrait for WsConnection {
    fn get_more_data(&mut self, layer_list: Layers, _data: &mut Data) -> Result<(), Vec<TramexError>> {
        // Don't send request if already waiting for a response
        if self.waiting_for_response {
            return Ok(());
        }
        
        let msg = LogGet::new(self.msg_id, layer_list, self.asking_size_max);
        log::debug!("📤 Sending log_get request #{}", self.msg_id);
        match serde_json::to_string(&msg) {
            Ok(msg_stringed) => {
                log::debug!("📤 JSON payload: {}", msg_stringed);
                self.ws_sender.send(WsMessage::Text(msg_stringed));
                self.msg_id += 1;
                self.waiting_for_response = true;
                log::debug!("⏳ Waiting for response...");
            }
            Err(err) => {
                log::error!("Error encoding message: {err:?}");
                return Err(vec![tramex_error!(
                    err.to_string(),
                    crate::errors::ErrorCode::WebSocketErrorEncodingMessage
                )]);
            }
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), TramexError> {
        self.close_impl()
    }
    
    fn supports_preloading(&self) -> bool {
        false  // WebSocket cannot preload - server controls data
    }
    
    fn get_total_event_count(&self) -> Option<usize> {
        None  // Unknown for WebSocket
    }
    
    fn is_fully_read(&self) -> bool {
        !self.available  // If connection is closed, we're done
    }
}

impl WsConnection {
    /// Check if we should send a new request
    /// Returns true if auto-loading is enabled and not currently waiting for a response
    /// The server-side timeout and allow_empty=false handle the timing automatically
    pub fn should_request_more(&self) -> bool {
        self.auto_loading && !self.waiting_for_response
    }
    
    /// Try to receive data
    /// # Errors
    /// Return an error if the data is not received correctly
    pub fn try_recv(&mut self, data: &mut Data) -> Result<(), Vec<TramexError>> {
        while let Some(event) = self.ws_receiver.try_recv() {
            // log::debug!("🔵 WebSocket event received: {:?}", event);
            log::debug!("🔵 WebSocket event received");
            self.connecting = false;
            match event {
                WsEvent::Message(msg) => {
                    self.available = true;
                    match msg {
                        WsMessage::Text(event_text) => {
                            // log::debug!("📨 Raw WebSocket text message: {}", event_text);
                            let decoded: Result<WebSocketLog, serde_json::Error> = serde_json::from_str(&event_text);
                            match decoded {
                                Ok(decoded_data) => {
                                    log::debug!("✅ Received log_get response with {} logs", decoded_data.logs.len());
                                    
                                    // Mark that we received the response - ready for next request
                                    self.waiting_for_response = false;
                                    
                                    let mut errors = vec![];
                                    for one_log in decoded_data.logs {
                                        // Skip NR band combinations logs
                                        if Layer::RRC == one_log.layer && one_log.data.iter().any(|line| line.contains("NR band combinations")) {
                                            log::debug!("Skipping NR band combinations log");
                                            continue;
                                        }
                                        match one_log.extract_data() {
                                            Ok(trace) => {
                                                data.events.push(trace);
                                            }
                                            Err(err) => {
                                                log::error!("Error while extracting data: {err:?}");
                                                errors.push(err);
                                            }
                                        }
                                    }
                                    if !errors.is_empty() {
                                        return Err(errors);
                                    }
                                }
                                Err(_err) => {
                                    let decoded_base: Result<BaseMessage, serde_json::Error> =
                                        serde_json::from_str(&event_text);
                                    match decoded_base {
                                        Ok(decoded_data) => {
                                            if decoded_data.message == "ready" {
                                                log::debug!("✅ Received 'ready' message from server: {}", decoded_data.name);
                                                log::debug!("💡 Server is ready. You need to click 'Load More' or enable auto-loading to request logs.");
                                            }
                                            log::debug!("📥 Received BaseMessage: {decoded_data:?}");
                                            self.name = decoded_data.name;
                                        }
                                        Err(err) => {
                                            log::error!("Error decoding message: {err:?}");
                                            log::error!("Message: {event_text:?}");
                                            return Err(vec![tramex_error!(
                                                err.to_string(),
                                                crate::errors::ErrorCode::WebSocketErrorDecodingMessage
                                            )]);
                                        }
                                    }
                                }
                            }
                        }
                        WsMessage::Unknown(str_error) => {
                            log::error!("Unknown message: {str_error:?}");
                            return Err(vec![tramex_error!(
                                str_error,
                                crate::errors::ErrorCode::WebSocketUnknownMessageReceived
                            )]);
                        }
                        WsMessage::Binary(bin) => {
                            log::error!("Unknown binary message: {bin:?}");
                            return Err(vec![tramex_error!(
                                format!("Unknown binary message: {bin:?}"),
                                crate::errors::ErrorCode::WebSocketUnknownBinaryMessageReceived
                            )]);
                        }
                        _ => {
                            log::debug!("Received Ping-Pong")
                        }
                    }
                }
                WsEvent::Opened => {
                    self.available = true;
                    log::debug!("✅ WebSocket connection opened successfully");
                }
                WsEvent::Closed => {
                    self.available = false;
                    log::debug!("WebSocket closed");
                    return Err(vec![tramex_error!(
                        "WebSocket closed".to_string(),
                        crate::errors::ErrorCode::WebSocketClosed
                    )]);
                }
                WsEvent::Error(str_err) => {
                    self.available = false;
                    log::error!("WebSocket error: {str_err:?}");
                    return Err(vec![tramex_error!(str_err, crate::errors::ErrorCode::WebSocketError)]);
                }
            }
        }
        Ok(())
    }
}

impl Debug for WsConnection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Interface")
            .field("ws_sender", &"Box<WsSender>")
            .field("ws_receiver", &"Box<WsReceiver>")
            .field("connecting", &self.connecting)
            .finish()
    }
}

impl Drop for WsConnection {
    fn drop(&mut self) {
        log::debug!("Cleaning WsConnection");
        self.ws_sender.close()
    }
}
