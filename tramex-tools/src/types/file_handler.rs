use crate::functions::extract_hexe;
use crate::types::internals::MessageType;
use crate::types::internals::Trace;
use crate::types::websocket_types::Direction::{DL, UL};
use regex::Regex;

const RGX: &str = r"(?mi)(?<timestamp>\d{2}:\d{2}:\d{2}\.\d{3})\s+\[(?<msgtype>.*?)\]\s(?<direction>\w+)\s*-\s*(?<id>\d{2})\s*(?<canal>(?:\w+)-?(?:\w*)):\s(?<messagecanal>(?:\w|\s)+)$(?<hexa>(?:\s+(?:\d\d\d\d):\s+(?:(?:(?:(?:[0-9a-f]+)\s{1,2}))*).*$)*)";

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
        for (_, [timestamp, msgtype, direction, _id, canal, messagecanal, hexa]) in
            rgx.captures_iter(&hay).map(|c| c.extract())
        {
            let dir = if direction == "DL" { DL } else { UL };
            let m = MessageType {
                timestamp: timestamp.to_owned(), // TODO use u64 and convert to float timestamp
                msgtype: msgtype.to_owned(),
                direction: dir,
                canal: canal.to_owned(),
                canal_msg: messagecanal.to_owned(),
            };
            let splitted = hexa.split("\n").collect();
            let bytes: Vec<u8> = if let Some(byte) = extract_hexe(&splitted) {
                byte
            } else {
                vec![]
            };
            vtraces.push(Trace {
                trace_type: m,
                hexa: bytes,
            });
        }
        return vtraces;
    }
}
