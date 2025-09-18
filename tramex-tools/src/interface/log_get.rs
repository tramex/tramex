//! LogGet struct for sending log_get message to the server
use crate::interface::layer::Layers;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
/// LogGet struct
pub struct LogGet {
    /// Timeout
    timeout: u64,

    /// Minimum size of the log
    min: u64,

    /// Maximum size of the log
    max: u64,

    /// Layers
    layers: Layers,

    /// Message
    message: String,

    /// Headers
    headers: bool,

    /// Message ID
    message_id: u64,
}

impl LogGet {
    /// Create a new LogGet struct
    pub fn new(id: u64, layers_list: Layers, max_size: u64) -> Self {
        let max_size = max_size.clamp(64, 4096);
        Self {
            timeout: 1,
            min: 64,
            max: max_size,
            layers: layers_list,
            message: "log_get".to_owned(),
            headers: false,
            message_id: id,
        }
    }
}
