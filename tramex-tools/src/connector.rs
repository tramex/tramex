use crate::functions::extract_hexe;
use crate::types::file_handler::File;
use crate::types::internals::{Data, Interface, MessageType, Trace};
use crate::types::websocket_types::{WebSocketLog, WsConnection, WsEvent, WsMessage};

pub struct Connector {
    pub interface: Interface,
    pub data: Data,
}

impl Connector {
    pub fn new_ws(ws: WsConnection) -> Self {
        Self {
            interface: Interface::Ws(ws),
            data: Data::default(),
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
        }
    }

    pub fn try_recv(&mut self) {
        match &mut self.interface {
            Interface::Ws(ref mut ws) => {
                while let Some(event) = ws.ws_receiver.try_recv() {
                    match event {
                        WsEvent::Message(msg) => match msg {
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
                                ws.error = true;
                                log::error!("Unknown message: {:?}", str_error);
                                ws.errorstr = str_error;
                            }
                            WsMessage::Binary(bin) => {
                                ws.error = true;
                                ws.errorstr = format!("{:?}", bin);
                            }
                            _ => {
                                ws.error = true;
                                ws.errorstr = "Received Ping-Pong".to_string();
                            }
                        },
                        WsEvent::Opened => {
                            ws.connected = true;
                        }
                        WsEvent::Closed => {
                            ws.connected = false;
                        }
                        WsEvent::Error(str_err) => {
                            ws.connected = false;
                            ws.error = true;
                            log::error!("Unknown message: {:?}", str_err);
                            ws.errorstr = str_err;
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
            }
        }
    }
}
