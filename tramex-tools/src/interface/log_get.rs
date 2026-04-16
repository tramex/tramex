//! LogGet struct for sending log_get message to the server
use crate::interface::layer::Layers;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
/// LogGet struct
pub struct LogGet {
    /// Message
    message: String,

    /// Message ID
    message_id: u64,

    /// Timeout in seconds - server waits this long for logs before responding
    timeout: u64,

    /// Minimum size of the log
    min: u64,

    /// Maximum size of the log
    max: u64,

    /// Layers
    layers: Layers,

    /// Headers
    headers: bool,

    /// Start timestamp
    start_timestamp: i64,
}

impl LogGet {
    /// Create a new LogGet struct
    /// Note: layers_list parameter is ignored - we always request ALL layers
    /// The filtering is done by the Application, not by the server
    pub fn new(id: u64, _layers_list: Layers, max_size: u64) -> Self {
        let max_size = max_size.clamp(64, 4096);
        const HOURS_TO_FETCH: i64 = 12;
        Self {
            message: "log_get".to_owned(),
            message_id: id,
            timeout: 1,
            min: 64,
            max: max_size,
            layers: Layers::all_debug(), // Always request all layers
            headers: false,
            start_timestamp: chrono::Utc::now().timestamp() - HOURS_TO_FETCH * 3600, // only fetch last 12h
        }
    }
}
