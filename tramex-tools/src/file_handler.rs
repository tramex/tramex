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

const RGX: &str = r"(?mi)(?<timestamp>\d{2}:\d{2}:\d{2}\.\d{3})\s+\[(?<layer>.*?)\]\s(?<direction>\w+)\s*-\s*(?<id>\d{2})\s*(?<canal>(?:\w+)-?(?:\w*)):\s(?<messagecanal>(?:\w|\s)+)$(?<hexa>(?:\s+(?:\d\d\d\d):\s+(?:(?:(?:(?:[0-9a-f]+)\s{1,2}))*).*$)*)";
const DEFAULT_NB: usize = 5;
#[derive(Debug, Clone)]
pub struct File {
    pub file_path: PathBuf,
    pub file_content: String,
    pub readed: bool,
    nb_read: usize,
    ix: usize,
}

impl Default for File {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from(""),
            file_content: "".to_string(),
            readed: false,
            nb_read: DEFAULT_NB,
            ix: 0,
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
    pub fn new(file_path: PathBuf, file_content: String) -> Self {
        Self {
            file_path,
            file_content,
            readed: false,
            nb_read: DEFAULT_NB,
            ix: 0,
        }
    }
    pub fn change_nb_read(&mut self, toread: usize) {
        self.nb_read = toread;
    }
    pub fn process(&mut self) -> Result<Vec<Trace>, TramexError> {
        let a = File::process_string(&self.file_content, self.nb_read, &mut self.ix);
        eprintln!("Ix {:}",self.ix);
        return a;
    }
    pub fn process_string(
        hay: &String,
        nb_to_read: usize,
        ix: &mut usize,
    ) -> Result<Vec<Trace>, TramexError> {
        let mut vtraces: Vec<Trace> = vec![];
        let lines: Vec<&str> = hay.lines().collect();
        for _ in 0..nb_to_read {
            let mtype: MessageType = Self::parse_line(lines[*ix]).unwrap();
            *ix += 1;
            let mut hex_str: Vec<&str> = vec![];
            while lines[*ix].trim_start().chars().next().unwrap() != '{' {
                hex_str.push(lines[*ix]);
                *ix += 1;
            }
            let hex = extract_hexe(&hex_str);
            vtraces.push(Trace {
                trace_type: mtype,
                hexa: hex,
            });
            let mut end = false;
            let mut brackets: i16 = 0;
            while !end {
                brackets = brackets + Self::count_brackets(lines[*ix]);
                *ix += 1;
                if brackets == 0 {
                    end = true;
                }
            }
            *ix += 1;
        }
        return Ok(vtraces);
    }
    pub fn count_brackets(hay: &str) -> i16 {
        let mut count: i16 = 0;
        for ch in hay.chars() {
            match ch {
                '{' => count += 1,
                '}' => count -= 1,
                _ => (),
            }
        }
        return count;
    }
    pub fn parse_line(line: &str) -> Result<MessageType, TramexError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(TramexError::new(
                "Error".to_string(),
                crate::errors::ErrorCode::WebSocketErrorEncodingMessage,
            ));
        }
        let date = match chrono::NaiveTime::parse_from_str(parts[0], "%H:%M:%S%.3f") {
            Ok(rdate) => rdate,
            Err(_) => {
                return Err(TramexError::new(
                    "Error while parsing date".to_string(),
                    crate::errors::ErrorCode::WebSocketErrorEncodingMessage,
                ));
            }
        };
        let layer = parts[1].trim_start_matches('[').trim_end_matches(']');
        let direction = parts[2];
        let binding = parts[5..].join(" ");
        let concatenated: Vec<&str> = binding.split(":").collect();
        return Ok(MessageType {
            timestamp: time_to_milliseconds(&date) as u64,
            layer: Layer::from_str(layer).unwrap_or_default(),
            direction: Direction::from_str(direction).unwrap_or_default(),
            canal: concatenated[0].to_owned(),
            canal_msg: concatenated[1].trim_start().to_owned(),
        });
    }
}
#[derive(Debug, Clone)]
pub struct Pos {
    pub start: usize,
    pub end: usize,
}
