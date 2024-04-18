use crate::functions::extract_hexe;
use crate::types::internals::MessageType;
use crate::types::internals::Trace;
use crate::types::websocket_types::Direction::{DL, UL};
use chrono::NaiveTime;
use chrono::Timelike;
use regex::Regex;

use super::websocket_types::Layer;

const RGX: &str = r"(?mi)(?<timestamp>\d{2}:\d{2}:\d{2}\.\d{3})\s+\[(?<layer>.*?)\]\s(?<direction>\w+)\s*-\s*(?<id>\d{2})\s*(?<canal>(?:\w+)-?(?:\w*)):\s(?<messagecanal>(?:\w|\s)+)$(?<hexa>(?:\s+(?:\d\d\d\d):\s+(?:(?:(?:(?:[0-9a-f]+)\s{1,2}))*).*$)*)";

#[derive(Debug)]
pub struct File {
    pub file_path: String,
    pub file_content: String,
    pub readed: bool,
}

impl Default for File {
    fn default() -> Self {
        Self {
            file_path: "".to_string(),
            file_content: "".to_string(),
            readed: false,
        }
    }
}
fn time_to_milliseconds(time: &NaiveTime) -> i64 {
    let hours_in_ms = time.hour() as i64 * 3600_000;
    let minutes_in_ms = time.minute() as i64 * 60_000;
    let seconds_in_ms = time.second() as i64 * 1000;
    let milliseconds = time.nanosecond() as i64 / 1_000_000; // convert nanoseconds to milliseconds

    hours_in_ms + minutes_in_ms + seconds_in_ms + milliseconds
}

impl File {
    pub fn new(file_path: String, file_content: String) -> Self {
        Self {
            file_path,
            file_content,
            readed: true,
        }
    }

    pub fn process_string(hay: &String) -> Vec<Trace> {
        //A FAIRE Compile Regex only one time
        let rgx = Regex::new(RGX).unwrap();
        let mut vtraces: Vec<Trace> = vec![];
        for (_, [timestamp, layer, direction, _id, canal, messagecanal, hexa]) in
            rgx.captures_iter(&hay).map(|c| c.extract())
        {
            let dir = if direction == "DL" { DL } else { UL };
            let date =
                chrono::NaiveTime::parse_from_str(timestamp, "%H:%M:%S%.3f").unwrap_or_default();
            let date = time_to_milliseconds(&date) as u64;
            use std::str::FromStr;
            let m = MessageType {
                timestamp: date,
                layer: Layer::from_str(layer).unwrap_or(Layer::None),
                direction: dir,
                canal: canal.to_owned(),
                canal_msg: messagecanal.to_owned(),
            };
            let splitted = hexa.split("\n").collect();
            let bytes: Vec<u8> = extract_hexe(&splitted);
            vtraces.push(Trace {
                trace_type: m,
                hexa: bytes,
            });
        }
        return vtraces;
    }
}
