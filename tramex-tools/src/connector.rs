use std::borrow::BorrowMut;

use crate::functions::extract_hexe;
use crate::types::file_handler::File;
use crate::types::internals::{Data, Interface, MessageType, Trace};
use crate::types::websocket_types::{WebSocketLog, WsConnection};
use ewebsock::{WsEvent, WsMessage};

pub struct Connector {
    pub interface: Interface,
    pub data: Data,
    pub available: bool,
}

impl Connector {
    pub fn new_ws(ws: WsConnection) -> Self {
        Self {
            interface: Interface::Ws(ws),
            data: Data::default(),
            available: false,
        }
    }
    pub fn new_file(file_path: String) -> Self {
        Self {
            interface: Interface::File(File {
                file_path,
                file_content: String::new(),
                readed: false,
                ..Default::default()
            }),
            data: Data::default(),
            available: false,
        }
    }
    pub fn new_file_content(file_path: String, file_content: String) -> Self {
        Self {
            interface: Interface::File(File {
                file_path,
                file_content,
                readed: false,
            }),
            data: Data::default(),
            available: true,
        }
    }

    pub fn try_recv(&mut self) {
        match &mut self.interface {
            Interface::Ws(ref mut ws) => {
                while let Some(event) = ws.ws_receiver.try_recv() {
                    match event {
                        WsEvent::Message(msg) => {
                            self.available = true;
                            match msg {
                                WsMessage::Text(event_text) => {
                                    let decoded: Result<WebSocketLog, serde_json::Error> =
                                        serde_json::from_str(&event_text);
                                    if let Ok(decoded) = decoded {
                                        for one_log in decoded.logs {
                                            let msg_type = MessageType {
                                                timestamp: one_log.timestamp.to_string(), // TODO use u64
                                                msgtype: "TODO".to_string(), // TODO use u64
                                                direction: one_log.dir.unwrap(), // TODO use u64
                                                canal: "TODO".to_string(),   // TODO use u64
                                                canal_msg: "TODO".to_string(), // TODO use u64
                                            };
                                            let hexa = extract_hexe(&one_log.data);
                                            let trace = Trace {
                                                trace_type: msg_type,
                                                hexa: hexa.unwrap(),
                                            };
                                            self.data.events.push(trace);
                                        }
                                    }
                                }
                                WsMessage::Unknown(str_error) => {
                                    log::error!("Unknown message: {:?}", str_error);
                                    ws.error_str = Some(str_error);
                                }
                                WsMessage::Binary(bin) => {
                                    ws.error_str = Some(format!("{:?}", bin));
                                }
                                _ => {
                                    ws.error_str = Some("Received Ping-Pong".to_string());
                                }
                            }
                        }
                        WsEvent::Opened => {
                            self.available = true;
                        }
                        WsEvent::Closed => {
                            self.available = false;
                        }
                        WsEvent::Error(str_err) => {
                            self.available = false;
                            log::error!("Unknown message: {:?}", str_err);
                            ws.error_str = Some(str_err);
                        }
                    }
                }
            }
            Interface::File(ref mut file) => {
                if file.readed {
                    return;
                }
                //TODO Un buffer ou on recupere une partie du fichier ?
                let processed = &mut File::process_string(&file.file_content);
                self.data.events.append(processed);
                file.readed = true;
                self.available = true;
            }
        }
    }
}
