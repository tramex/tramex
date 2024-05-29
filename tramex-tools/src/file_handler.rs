//! File Handler

use crate::data::MessageType;
use crate::data::Trace;
use crate::errors::TramexError;
use crate::functions::extract_hexe;
use crate::websocket::types::Direction;
use chrono::NaiveTime;
use chrono::Timelike;
use regex::Regex;
use std::path::PathBuf;
use std::str::FromStr;

use crate::websocket::layer::Layer;

/// Regular expression to extract the data from the file.
const RGX: &str = r"(?mi)(?<timestamp>\d{2}:\d{2}:\d{2}\.\d{3})\s+\[(?<layer>.*?)\]\s(?<direction>\w+)\s*-\s*(?<id>\d{2})\s*(?<canal>(?:\w+)-?(?:\w*)):\s(?<messagecanal>(?:\w|\s)+)$(?<hexa>(?:\s+(?:\d\d\d\d):\s+(?:(?:(?:(?:[0-9a-f]+)\s{1,2}))*).*$)*)";

#[derive(Debug, Clone)]
/// Data structure to store the file.
pub struct File {
    /// Path of the file.
    pub file_path: PathBuf,

    /// Content of the file.
    pub file_content: String,

    /// Readed status of the file.
    pub readed: bool,
}

impl Default for File {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from(""),
            file_content: "".to_string(),
            readed: false,
        }
    }
}

/// Convert a time to milliseconds.
fn time_to_milliseconds(time: &NaiveTime) -> i64 {
    let hours_in_ms = time.hour() as i64 * 3_600_000;
    let minutes_in_ms = time.minute() as i64 * 60_000;
    let seconds_in_ms = time.second() as i64 * 1000;
    let milliseconds = time.nanosecond() as i64 / 1_000_000; // convert nanoseconds to milliseconds

    hours_in_ms + minutes_in_ms + seconds_in_ms + milliseconds
}

impl File {
    /// Create a new file.
    pub fn new(file_path: PathBuf, file_content: String) -> Self {
        Self {
            file_path,
            file_content,
            readed: false,
        }
    }

    /// Read the file.
    /// # Errors
    /// Return an error if the file contains errors
    pub fn process(&self) -> Result<Vec<Trace>, TramexError> {
        File::process_string(&self.file_content)
    }

    /// Process the string of the file.
    /// # Errors
    /// Return an error if the file contains errors
    pub fn process_string(hay: &str) -> Result<Vec<Trace>, TramexError> {
        let rgx = if let Ok(re) = Regex::new(RGX) {
            re
        } else {
            return Err(TramexError::new(
                "Can't create Regex".to_string(),
                crate::errors::ErrorCode::default(),
            ));
        };
        let mut vtraces: Vec<Trace> = vec![];
        for (_, [timestamp, layer, direction, _id, canal, message_canal, hexa]) in
            rgx.captures_iter(hay).map(|c| c.extract())
        {
            let date =
                chrono::NaiveTime::parse_from_str(timestamp, "%H:%M:%S%.3f").unwrap_or_default();

            vtraces.push(Trace {
                trace_type: MessageType {
                    timestamp: time_to_milliseconds(&date) as u64,
                    layer: Layer::from_str(layer).unwrap_or_default(),
                    direction: Direction::from_str(direction).unwrap_or_default(),
                    canal: canal.to_owned(),
                    canal_msg: message_canal.to_owned(),
                },
                hexa: extract_hexe(&hexa.split('\n').collect::<Vec<&str>>()).unwrap_or_default(), // TODO handle
            });
        }
        if vtraces.is_empty() {
            return Err(TramexError::new(
                "Can't find Trace in File".to_string(),
                crate::errors::ErrorCode::WebSocketErrorEncodingMessage,
            ));
        }
        Ok(vtraces)
    }
}
