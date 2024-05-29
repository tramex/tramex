use crate::data::MessageType;
use crate::data::Trace;
use crate::errors::TramexError;
use crate::functions::extract_hexe;
use crate::websocket::types::Direction;
use chrono::NaiveTime;
use chrono::Timelike;
use std::path::PathBuf;
use std::str::FromStr;

use crate::websocket::layer::Layer;

const DEFAULT_NB: usize = 6;
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
    pub fn new_toread(file_path: PathBuf, file_content: String, nb_to_read: usize) -> Self {
        Self {
            file_path,
            file_content,
            readed: false,
            nb_read: nb_to_read,
            ix: 0,
        }
    }
    pub fn change_nb_read(&mut self, toread: usize) {
        self.nb_read = toread;
    }
    pub fn process(&mut self) -> (Vec<Trace>, Option<TramexError>) {
        let (vec_trace, opt_err) = File::process_string(&self.file_content, self.nb_read, &mut self.ix);
        if opt_err.is_some() {
            self.readed = true;
        }
        return (vec_trace, opt_err);
    }
    pub fn process_string(hay: &String, nb_to_read: usize, mut ix: &mut usize) -> (Vec<Trace>, Option<TramexError>) {
        let mut vtraces: Vec<Trace> = vec![];
        let lines: Vec<&str> = hay.lines().collect();
        for _ in 0..nb_to_read {
            match Self::parse_bloc(&lines, &mut ix) {
                Ok(trace) => {
                    vtraces.push(trace);
                }
                Err(err) => match &err {
                    Some(e) => {
                        eprintln!("Error {:} at line {:} : \n {:}", e.message, *ix, lines[*ix]);
                        return (vtraces, Some(e.clone()));
                    }
                    None => {
                        return (vtraces, Some(Self::eof_error()));
                    }
                },
            };
        }
        return (vtraces, None);
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
    fn parse_bloc(lines: &Vec<&str>, ix: &mut usize) -> Result<Trace, Option<TramexError>> {
        let lines_len = lines.len();
        if (lines_len as i32 - *ix as i32) < 3 {
            return Err(None);
        }
        let mtype = match Self::parse_line(lines[*ix]) {
            Ok(m) => m,
            Err(e) => {
                return Err(Some(e));
            }
        };
        *ix += 1;
        let mut hex_str: Vec<&str> = vec![];
        while *ix < lines_len {
            match lines[*ix].trim_start().chars().next() {
                Some(c) => {
                    if c == '{' {
                        break;
                    }
                }
                None => {
                    break;
                }
            }
            hex_str.push(lines[*ix]);
            *ix += 1;
        }
        if *ix >= lines_len {
            *ix = *ix - 1;
            return Err(Some(Self::parsing_error(
                "Could not find the end of the hexadecimal".to_string(),
            )));
        }
        let hex = extract_hexe(&hex_str); //TODO do real error returning
        let trace = Trace {
            trace_type: mtype,
            hexa: hex,
        };
        let mut end = false;
        let mut brackets: i16 = 0;
        while (*ix < lines_len) && !end {
            brackets = brackets + Self::count_brackets(lines[*ix]);
            *ix += 1;
            if brackets == 0 {
                end = true;
            }
        }
        if *ix >= lines_len && !end {
            *ix = *ix - 1;
            return Err(Some(Self::parsing_error(
                "Could not parse the JSON like part, missing closing }".to_string(),
            )));
        }
        *ix += 1;
        return Ok(trace);
    }
    fn parse_line(line: &str) -> Result<MessageType, TramexError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(Self::parsing_error("Could not find enough (5) parameters".to_string()));
        }
        let date = match chrono::NaiveTime::parse_from_str(parts[0], "%H:%M:%S%.3f") {
            Ok(rdate) => rdate,
            Err(_) => {
                return Err(Self::parsing_error("Error while parsing date".to_string()));
            }
        };
        let layer_result: Result<Layer, ()> = Layer::from_str(parts[1].trim_start_matches('[').trim_end_matches(']'));
        let direction_result = Direction::from_str(parts[2]);
        let binding: String = parts[5..].join(" ");
        let concatenated: Vec<&str> = binding.split(":").collect();
        let layer: Layer = match layer_result {
            Ok(l) => l,
            Err(_) => {
                return Err(Self::parsing_error("The layer could not be parsed".to_string()));
            }
        };
        eprintln!("{:?}", layer);
        if layer == Layer::None {
            return Err(Self::parsing_error("The layer could not be parsed".to_string()));
        }
        let direction = match direction_result {
            Ok(d) => d,
            Err(_) => return Err(Self::parsing_error("The direction could not be parsed".to_string())),
        };
        if concatenated.len() < 2 || concatenated[0].is_empty() || concatenated[1].is_empty() {
            return Err(Self::parsing_error(
                "The canal and/or canal message could not be parsed".to_string(),
            ));
        }
        return Ok(MessageType {
            timestamp: time_to_milliseconds(&date) as u64,
            layer: layer,
            direction: direction,
            canal: concatenated[0].to_owned(),
            canal_msg: concatenated[1].trim_start().to_owned(),
        });
    }
    fn parsing_error(message: String) -> TramexError {
        TramexError::new(message, crate::errors::ErrorCode::FileParsing)
    }
    fn eof_error() -> TramexError {
        TramexError::new("End of file".to_string(), crate::errors::ErrorCode::EndOfFile)
    }
}
#[derive(Debug, Clone)]
pub struct Pos {
    pub start: usize,
    pub end: usize,
}
