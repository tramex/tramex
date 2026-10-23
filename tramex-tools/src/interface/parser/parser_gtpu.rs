//! Parser for GTPU traces
use super::ParsingError;
use super::hex_extractor::extract_binary_from_lines;
use crate::data::{AdditionalInfos, Trace};
use crate::interface::association::TraceRelation;
use std::str::FromStr;

use crate::interface::{layer::Layer, types::Direction};

use super::{FileParser, LayerParser, ParsedHeader};

#[derive(Debug, Clone, Default)]
/// Data structure to store the GTPU message type
pub struct GTPUInfos {
    /// Direction of the message (TO/FROM)
    pub direction: Direction,

    /// Message type (e.g., "G-PDU", "Echo Request")
    pub message_type: String,

    /// Connection info (e.g., "127.0.1.100:2152")
    pub connection_info: Option<String>,

    /// Tunnel Endpoint Identifier
    pub teid: Option<u32>,

    /// QoS Flow Identifier
    pub qfi: Option<u8>,

    /// Length of the encapsulated SDU
    pub sdu_len: Option<u32>,

    /// Source address of the encapsulated packet (e.g., "192.168.3.2:51158")
    pub src_addr: Option<String>,

    /// Destination address of the encapsulated packet (e.g., "172.217.18.202:443")
    pub dst_addr: Option<String>,
}

impl GTPUInfos {
    /// Address of the remote (data network) side of the encapsulated flow
    pub fn remote_addr(&self) -> Option<&String> {
        match self.direction {
            Direction::FROM | Direction::DL => self.src_addr.as_ref(),
            _ => self.dst_addr.as_ref(),
        }
    }

    /// Address of the UE side of the encapsulated flow
    pub fn ue_addr(&self) -> Option<&String> {
        match self.direction {
            Direction::FROM | Direction::DL => self.dst_addr.as_ref(),
            _ => self.src_addr.as_ref(),
        }
    }
}

/// GTPU Parser
pub struct GTPUParser;

impl GTPUParser {
    /// Function that parses the remaining lines of a log
    fn parse_lines(lines: &[String]) -> Vec<String> {
        lines.to_vec()
    }

    /// Parse the GTPU payload (everything after the IP:port)
    /// e.g. "G-PDU TEID=0x5315caa3 QFI=1 SDU_len=1452: IP/TCP 192.168.3.2:51158 > 172.217.18.202:443"
    fn parse_payload(direction: Direction, connection_info: Option<String>, payload: &str) -> GTPUInfos {
        let (header, inner) = match payload.split_once(": ") {
            Some((h, i)) => (h, Some(i.trim())),
            None => (payload.trim_end_matches(':'), None),
        };

        let mut tokens = header.split_whitespace();
        let mut infos = GTPUInfos {
            direction,
            message_type: tokens.next().unwrap_or("Unknown").to_string(),
            connection_info,
            ..Default::default()
        };

        for token in tokens {
            let Some((key, value)) = token.trim_end_matches(':').split_once('=') else {
                continue;
            };
            match key {
                "TEID" => {
                    let hex = value.trim_start_matches("0x").trim_start_matches("0X");
                    infos.teid = u32::from_str_radix(hex, 16).ok();
                }
                "QFI" => infos.qfi = value.parse().ok(),
                "SDU_len" => infos.sdu_len = value.parse().ok(),
                _ => {}
            }
        }

        if let Some(inner) = inner {
            let (_, flow) = inner.split_once(' ').unwrap_or((inner, ""));
            if let Some((src, dst)) = flow.split_once(" > ") {
                infos.src_addr = Some(src.trim().to_string());
                infos.dst_addr = Some(dst.trim().to_string());
            }
        }

        infos
    }
}

impl FileParser for GTPUParser {
    fn parse_additional_infos(lines: &[String]) -> Result<AdditionalInfos, ParsingError> {
        let line = &lines[0];
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Example: "13:20:45.574 [GTPU] TO 127.0.1.100:2152 G-PDU TEID=0x5315caa3 QFI=1 SDU_len=1452: IP/TCP ..."
        // Parts: ["13:20:45.574", "[GTPU]", "TO", "127.0.1.100:2152", "G-PDU", ...]
        if parts.len() < 5 {
            return Err(ParsingError::new(
                "Could not find enough (5) parameters for GTPU".to_string(),
                1,
            ));
        }

        let direction_result = Direction::from_str(parts[2]);
        let direction = match direction_result {
            Ok(d) => d,
            Err(_) => {
                return Err(ParsingError::new(
                    format!("The direction could not be parsed in the part {:?} of {}", parts[2], line),
                    1,
                ));
            }
        };

        // Connection info is typically at parts[3] (IP:port)
        let (connection_info, payload_start) = if parts[3].contains(':') {
            (Some(parts[3].to_string()), 4)
        } else {
            (None, 3)
        };

        let payload = parts[payload_start..].join(" ");
        Ok(AdditionalInfos::GTPUInfos(Self::parse_payload(
            direction,
            connection_info,
            &payload,
        )))
    }

    fn parse(lines: &[String]) -> Result<Trace, ParsingError> {
        let additional_infos = match Self::parse_additional_infos(lines) {
            Ok(m) => m,
            Err(e) => {
                return Err(e);
            }
        };

        let text = Self::parse_lines(lines);

        let binary = extract_binary_from_lines(lines);

        // Parse timestamp from first line
        let timestamp = if let Some(first_line) = lines.first() {
            super::parse_timestamp(first_line)?
        } else {
            0
        };

        let trace = Trace {
            timestamp,
            layer: Layer::GTPU,
            additional_infos,
            text: Some(text),
            binary,
            relation: TraceRelation::default(),
        };
        Ok(trace)
    }
}

impl LayerParser for GTPUParser {
    /// Parse GTPU payload lines.
    /// data_lines[0] = "IP:PORT G-PDU TEID=... QFI=... SDU_len=..."
    fn parse_layer(header: &ParsedHeader, data_lines: &[String]) -> Result<AdditionalInfos, ParsingError> {
        if data_lines.is_empty() {
            return Err(ParsingError::new("GTPU: empty data lines".to_string(), 0));
        }
        let first_line = data_lines[0].trim();

        // First part might be IP:port
        let (connection_info, payload) = match first_line.split_once(char::is_whitespace) {
            Some((first, rest)) if first.contains(':') && first.contains('.') => (Some(first.to_string()), rest),
            _ => (header.connection_info.clone(), first_line),
        };

        Ok(AdditionalInfos::GTPUInfos(Self::parse_payload(
            header.direction.clone(),
            connection_info,
            payload.trim(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gtpu_ul() {
        let line = "13:20:45.574 [GTPU] TO 127.0.1.100:2152 G-PDU TEID=0x5315caa3 QFI=1 SDU_len=1452: IP/TCP 192.168.3.2:51158 > 172.217.18.202:443";
        let AdditionalInfos::GTPUInfos(infos) = GTPUParser::parse_additional_infos(&[line.to_string()]).unwrap() else {
            panic!("Expected GTPUInfos");
        };
        assert_eq!(infos.direction, Direction::TO);
        assert_eq!(infos.message_type, "G-PDU");
        assert_eq!(infos.connection_info.as_deref(), Some("127.0.1.100:2152"));
        assert_eq!(infos.teid, Some(0x5315caa3));
        assert_eq!(infos.qfi, Some(1));
        assert_eq!(infos.sdu_len, Some(1452));
        assert_eq!(infos.ue_addr().map(String::as_str), Some("192.168.3.2:51158"));
        assert_eq!(infos.remote_addr().map(String::as_str), Some("172.217.18.202:443"));
    }

    #[test]
    fn test_parse_gtpu_dl_ipv6() {
        let line = "10:32:29.599 [GTPU] FROM 127.0.1.100:2152 G-PDU TEID=0xbe4c81c5 QFI=1 SDU_len=88: IP/ICMPV6 fe80::84b0:ba3e:79ea:d3c2 > ff02::1";
        let AdditionalInfos::GTPUInfos(infos) = GTPUParser::parse_additional_infos(&[line.to_string()]).unwrap() else {
            panic!("Expected GTPUInfos");
        };
        assert_eq!(infos.direction, Direction::FROM);
        assert_eq!(infos.teid, Some(0xbe4c81c5));
        assert_eq!(infos.remote_addr().map(String::as_str), Some("fe80::84b0:ba3e:79ea:d3c2"));
        assert_eq!(infos.ue_addr().map(String::as_str), Some("ff02::1"));
    }

    #[test]
    fn test_parse_gtpu_layer() {
        let header = ParsedHeader {
            direction: Direction::TO,
            ..Default::default()
        };
        let data =
            vec!["127.0.1.100:2152 G-PDU TEID=0x5315caa3 QFI=2 SDU_len=10: IP/UDP 1.2.3.4:5 > 6.7.8.9:10".to_string()];
        let AdditionalInfos::GTPUInfos(infos) = GTPUParser::parse_layer(&header, &data).unwrap() else {
            panic!("Expected GTPUInfos");
        };
        assert_eq!(infos.connection_info.as_deref(), Some("127.0.1.100:2152"));
        assert_eq!(infos.qfi, Some(2));
        assert_eq!(infos.remote_addr().map(String::as_str), Some("6.7.8.9:10"));
    }

    #[test]
    fn test_parse_gtpu_no_inner() {
        let line = "10:32:29.599 [GTPU] TO 127.0.1.100:2152 Echo Request";
        let AdditionalInfos::GTPUInfos(infos) = GTPUParser::parse_additional_infos(&[line.to_string()]).unwrap() else {
            panic!("Expected GTPUInfos");
        };
        assert_eq!(infos.message_type, "Echo");
        assert_eq!(infos.teid, None);
        assert_eq!(infos.remote_addr(), None);
    }
}
