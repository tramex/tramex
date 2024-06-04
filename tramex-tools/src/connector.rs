//! Connector module
use std::path::PathBuf;

use crate::data::Data;
#[cfg(feature = "websocket")]
use crate::data::MessageType;
#[cfg(feature = "websocket")]
use crate::data::Trace;
use crate::errors::{ErrorCode, TramexError};
use crate::file_handler::File;
use crate::interface::Interface;
use crate::websocket::layer::Layers;
#[cfg(feature = "websocket")]
use crate::websocket::log_get::LogGet;
#[cfg(feature = "websocket")]
use crate::websocket::types::WebSocketLog;
#[cfg(feature = "websocket")]
use crate::websocket::ws_connection::WsConnection;
#[cfg(feature = "websocket")]
use ewebsock::{WsEvent, WsMessage};
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(default)]
/// Connector
pub struct Connector {
    #[serde(skip)]
    /// Interface
    pub interface: Interface,

    #[serde(skip)]
    /// Data
    pub data: Data,

    #[serde(skip)]
    /// Available
    pub available: bool,

    /// Asking size max
    pub asking_size_max: u64,

    /// Url
    pub url: String,
}

impl Connector {
    /// Create a new Connector
    pub fn new() -> Self {
        Self {
            interface: Interface::None,
            data: Data::default(),
            available: false,
            asking_size_max: 1024,
            url: "ws://127.0.0.1:9001".to_owned(),
        }
    }

    /// Clear data of connector
    pub fn clear_data(&mut self) {
        self.data = Data::default();
        self.available = false;
    }

    /// Clear connector interface
    pub fn clear_interface(&mut self) {
        self.interface = Interface::None;
        self.available = false;
    }

    /// Connect to a websocket
    /// # Errors
    /// Return an error if the connection failed
    #[cfg(feature = "websocket")]
    pub fn connect(&mut self, url: &str, wakeup: impl Fn() + Send + Sync + 'static) -> Result<(), TramexError> {
        let options = ewebsock::Options::default();
        match ewebsock::connect_with_wakeup(url, options, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.interface = Interface::Ws(WsConnection {
                    ws_sender,
                    ws_receiver,
                    msg_id: 1,
                    connecting: true,
                });
                Ok(())
            }
            Err(error) => {
                log::error!("Failed to connect to {:?}: {}", url, error);
                Err(TramexError::new(
                    error.to_string(),
                    crate::errors::ErrorCode::WebSocketFailedToConnect,
                ))
            }
        }
    }

    /// Set file mode using a File
    pub fn set_file(&mut self, file: File) {
        log::debug!("Set file available");
        self.interface = Interface::File(file);
        self.available = true;
    }

    /// Set ws mode
    #[cfg(feature = "websocket")]
    pub fn new_ws(ws: WsConnection) -> Self {
        Self {
            #[cfg(feature = "websocket")]
            interface: Interface::Ws(ws),
            data: Data::default(),
            available: false,
            ..Default::default()
        }
    }

    /// set file mode using a path
    pub fn new_file(file_path: PathBuf) -> Self {
        Self {
            interface: Interface::File(File::new_with_to_read(file_path, String::new(), 50)),
            data: Data::default(),
            available: false,
            ..Default::default()
        }
    }

    /// set file mode using a path and content
    pub fn new_file_content(file_path: PathBuf, file_content: String) -> Self {
        Self {
            interface: Interface::File(File::new_with_to_read(file_path, file_content, 50)),
            data: Data::default(),
            available: true,
            ..Default::default()
        }
    }

    /// Get more data depending on the interface
    /// # Errors
    /// Return an error if the interface is not set
    pub fn get_more_data(&mut self, _layers_list: Layers) -> Result<(), TramexError> {
        log::debug!("Get more data");
        match &mut self.interface {
            #[cfg(feature = "websocket")]
            Interface::Ws(ref mut ws) => {
                let msg = LogGet::new(ws.msg_id, _layers_list, self.asking_size_max);
                match serde_json::to_string(&msg) {
                    Ok(msg_stringed) => {
                        log::debug!("{}", msg_stringed);
                        ws.ws_sender.send(WsMessage::Text(msg_stringed));
                        ws.msg_id += 1;
                    }
                    Err(err) => {
                        log::error!("Error encoding message: {:?}", err);
                        return Err(TramexError::new(
                            err.to_string(),
                            crate::errors::ErrorCode::WebSocketErrorEncodingMessage,
                        ));
                    }
                }
            }
            Interface::File(ref mut curr_file) => {
                if curr_file.full_read {
                    return Ok(());
                }
                let (m_vec, opt_err) = &mut curr_file.process();
                self.data.events.append(m_vec);
                self.available = true;
                match opt_err {
                    Some(err) => {
                        if !(matches!(err.code, ErrorCode::EndOfFile)) {
                            return Err(err.clone());
                        }
                    }
                    None => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Try to receive data
    /// # Errors
    /// Return an error if the interface is not set
    pub fn try_recv(&mut self) -> Result<(), TramexError> {
        match &mut self.interface {
            #[cfg(feature = "websocket")]
            Interface::Ws(ref mut ws) => {
                while let Some(event) = ws.ws_receiver.try_recv() {
                    ws.connecting = false;
                    match event {
                        WsEvent::Message(msg) => {
                            self.available = true;
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
                                                    text: one_log.data,
                                                };
                                                self.data.events.push(trace);
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
                            self.available = true;
                            log::debug!("WebSocket opened");
                        }
                        WsEvent::Closed => {
                            self.available = false;
                            log::debug!("WebSocket closed");
                            return Err(TramexError::new(
                                "WebSocket closed".to_string(),
                                crate::errors::ErrorCode::WebSocketClosed,
                            ));
                        }
                        WsEvent::Error(str_err) => {
                            self.available = false;
                            log::error!("WebSocket error: {:?}", str_err);
                            return Err(TramexError::new(str_err, crate::errors::ErrorCode::WebSocketError));
                        }
                    }
                }
            }
            //Do nothing if its a File & None
            Interface::File(_) => {}
            _ => {}
        }
        Ok(())
    }
}
