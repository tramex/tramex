use crate::file_handler::File;
use crate::types::internals::{Data, Interface, MessageType, Trace};
use crate::websocket::{
    layer::Layers, log_get::LogGet, types::WebSocketLog, ws_connection::WsConnection,
};
use ewebsock::{WsEvent, WsMessage};

#[derive(Debug, Default)]
pub struct Connector {
    pub interface: Interface,
    pub data: Data,
    pub available: bool,
    pub asking_size_max: u64,
}

impl Drop for Connector {
    fn drop(&mut self) {
        log::debug!("Cleaning connector");
        match &mut self.interface {
            Interface::Ws(ref mut ws) => {
                if let Err(err) = ws.ws_sender.close() {
                    log::error!("Error closing WebSocket: {}", err);
                }
            }
            Interface::File(ref mut file) => {
                file.file_content.clear();
            }
            _ => {}
        }
    }
}

impl Connector {
    pub fn new() -> Self {
        Self {
            interface: Interface::None,
            data: Data::default(),
            available: false,
            asking_size_max: 500,
        }
    }
    pub fn connect(
        &mut self,
        url: &str,
        wakeup: impl Fn() + Send + Sync + 'static,
    ) -> Result<(), String> {
        let options = ewebsock::Options::default();
        match ewebsock::connect_with_wakeup(url, options, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.interface = Interface::Ws(WsConnection {
                    ws_sender: Box::new(ws_sender),
                    ws_receiver: Box::new(ws_receiver),
                    connecting: true,
                    error_str: None,
                });
                Ok(())
            }
            Err(error) => {
                log::error!("Failed to connect to {:?}: {}", url, error);
                Err(error.to_string())
            }
        }
    }
    pub fn new_ws(ws: WsConnection) -> Self {
        Self {
            interface: Interface::Ws(ws),
            data: Data::default(),
            available: false,
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
        }
    }

    pub fn get_more_data(&mut self, msg_id: u64, layers_list: Layers) {
        log::info!("Get more data");
        match &mut self.interface {
            Interface::Ws(ref mut ws) => {
                let msg = LogGet::new(msg_id, layers_list, self.asking_size_max);
                if let Ok(msg_stringed) = serde_json::to_string(&msg) {
                    log::info!("{}", msg_stringed);
                    ws.ws_sender.send(WsMessage::Text(msg_stringed));
                }
            }
            Interface::File(ref mut _file) => {
                //TODO READ
            }
            _ => {}
        }
    }

    pub fn try_recv(&mut self) {
        match &mut self.interface {
            Interface::Ws(ref mut ws) => {
                while let Some(event) = ws.ws_receiver.try_recv() {
                    ws.connecting = false;
                    match event {
                        WsEvent::Message(msg) => {
                            self.available = true;
                            match msg {
                                WsMessage::Text(event_text) => {
                                    let decoded: Result<WebSocketLog, serde_json::Error> =
                                        serde_json::from_str(&event_text);
                                    match decoded {
                                        Ok(decoded_data) => {
                                            for one_log in decoded_data.logs {
                                                let canal_msg = one_log
                                                    .extract_canal_msg()
                                                    .unwrap_or("".to_owned());
                                                let hexa = one_log.extract_hexe();
                                                let msg_type = MessageType {
                                                    timestamp: one_log.timestamp.to_owned(),
                                                    layer: one_log.layer,
                                                    direction: one_log.dir.unwrap(),
                                                    canal: one_log.channel.unwrap_or_default(),
                                                    canal_msg: canal_msg,
                                                };
                                                let trace = Trace {
                                                    trace_type: msg_type,
                                                    hexa: hexa,
                                                };
                                                self.data.events.push(trace);
                                            }
                                        }
                                        Err(err) => {
                                            log::error!("Error decoding message: {:?}", err);
                                            log::error!("Message: {:?}", event_text);
                                            ws.error_str = Some(err.to_string());
                                        }
                                    }
                                }
                                WsMessage::Unknown(str_error) => {
                                    log::error!("Unknown message: {:?}", str_error);
                                    ws.error_str = Some(str_error);
                                }
                                WsMessage::Binary(bin) => {
                                    log::error!("Unknown binary message: {:?}", bin);
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
            _ => {}
        }
    }
}
