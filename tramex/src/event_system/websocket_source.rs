//! WebSocket data source implementation

use super::data_source::{DataSource, DataSourceType};
use tramex_tools::data::Trace;
use tramex_tools::data::Data;
use tramex_tools::interface::layer::Layers;
use tramex_tools::interface::websocket::ws_connection::WsConnection;
use tramex_tools::interface::interface_types::InterfaceTrait;
use tramex_tools::errors::{TramexError, ErrorCode};
use tramex_tools::tramex_error;
use std::any::Any;

/// WebSocket data source
pub struct WebSocketSource {
    /// WebSocket connection
    connection: WsConnection,
    
    /// URL of the WebSocket server
    url: String,
    /// Whether connection is established
    connected: bool,
    /// Auto-loading enabled
    #[allow(dead_code)]
    auto_loading: bool,
    /// Temporary data holder for parsing
    temp_data: Data,
}

impl WebSocketSource {
    /// Create a new WebSocket source from an existing connection
    pub fn new(url: String, connection: WsConnection) -> Self {
        Self {
            connection,
            url,
            connected: true,
            auto_loading: true,
            temp_data: Data::default(),
        }
    }
    
    /// Connect to WebSocket server
    pub fn connect(url: String, wakeup: impl Fn() + Send + Sync + 'static) -> Result<Self, Vec<TramexError>> {
        match WsConnection::connect(&url, wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                let connection = WsConnection::new(ws_sender, ws_receiver);
                Ok(Self::new(url, connection))
            }
            Err(error_msg) => {
                let error = tramex_error!(error_msg, ErrorCode::WebSocketFailedToConnect);
                Err(vec![error])
            }
        }
    }
    
    /// Get the underlying connection
    pub fn connection(&self) -> &WsConnection {
        &self.connection
    }
    
    /// Get mutable connection
    pub fn connection_mut(&mut self) -> &mut WsConnection {
        &mut self.connection
    }
}

impl DataSource for WebSocketSource {
    fn poll(&mut self) -> Result<Vec<Trace>, Vec<TramexError>> {
        // Try to receive any incoming messages
        self.temp_data.events.clear();
        self.connection.try_recv(&mut self.temp_data)?;
        
        // Extract and return the events
        Ok(self.temp_data.events.clone())
    }
    
    fn request_more(&mut self, layers: &Layers) -> Result<(), Vec<TramexError>> {
        // Send log_get request if not already waiting
        if self.connection.should_request_more() {
            self.connection.get_more_data(layers.clone(), &mut self.temp_data)?;
        }
        Ok(())
    }
    
    fn is_auto_loading(&self) -> bool {
        self.connection.auto_loading
    }
    
    fn toggle_auto_loading(&mut self) {
        self.connection.auto_loading = !self.connection.auto_loading;
        let status = if self.connection.auto_loading { "▶ Resumed" } else { "⏸ Paused" };
        log::info!("WebSocketSource: Auto-loading {}", status);
    }
    
    fn has_more(&self) -> bool {
        // WebSocket is a stream, always has potential for more data
        self.connected
    }
    
    fn progress(&self) -> Option<f32> {
        // WebSocket has no concept of progress
        None
    }
    
    fn source_type(&self) -> DataSourceType {
        DataSourceType::WebSocket {
            url: self.url.clone(),
            connected: self.connected,
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
