//! Connector module
use std::path::PathBuf;

use crate::data::Data;
use crate::errors::TramexError;
use crate::interface::interface_file::file_handler::File;
use crate::interface::interface_types::{Interface, InterfaceTrait};
use crate::interface::layer::Layers;
#[cfg(feature = "websocket")]
use crate::interface::websocket::ws_connection::WsConnection;
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(default)]
/// Connector
pub struct Connector {
    #[serde(skip)]
    /// Interface
    pub interface: Option<Interface>,

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
            interface: None,
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
        self.interface = None;
        self.available = false;
    }

    /// Connect to a websocket
    /// # Errors
    /// Return an error if the connection failed
    #[cfg(feature = "websocket")]
    pub fn connect(&mut self, url: &str, wakeup: impl Fn() + Send + Sync + 'static) -> Result<(), TramexError> {
        match WsConnection::connect(url, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.interface = Some(Interface::Ws(WsConnection {
                    ws_sender,
                    ws_receiver,
                    msg_id: 1,
                    connecting: true,
                }));
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
        self.interface = Some(Interface::File(file));
        self.available = true;
    }

    /// Set ws mode
    #[cfg(feature = "websocket")]
    pub fn new_ws(ws: WsConnection) -> Self {
        Self {
            #[cfg(feature = "websocket")]
            interface: Some(Interface::Ws(ws)),
            data: Data::default(),
            available: false,
            ..Default::default()
        }
    }

    /// set file mode using a path
    pub fn new_file(file_path: PathBuf) -> Self {
        Self {
            interface: Some(Interface::File(File::new(file_path, String::new()))),
            data: Data::default(),
            available: false,
            ..Default::default()
        }
    }

    /// set file mode using a path and content
    pub fn new_file_content(file_path: PathBuf, file_content: String) -> Self {
        Self {
            interface: Some(Interface::File(File::new(file_path, file_content))),
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
            Some(inter) => inter.get_more_data(_layers_list, self.asking_size_max, &mut self.data, &mut self.available),
            None => {
                log::debug!("Error: Interface not set");
                Ok(())
            }
        }
    }

    /// Try to receive data
    /// # Errors
    /// Return an error if the interface is not set
    pub fn try_recv(&mut self) -> Result<(), TramexError> {
        match &mut self.interface {
            Some(inter) => inter.try_recv(&mut self.data, &mut self.available),
            None => {
                log::debug!("Error: Interface not set");
                Ok(())
            }
        }
    }
}
