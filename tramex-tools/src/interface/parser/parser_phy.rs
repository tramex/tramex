//! PHY layer parser for PDSCH/PUSCH traces

use super::ParsingError;
use super::hex_extractor::extract_binary_from_lines;
use crate::interface::association::TraceRelation;
use crate::data::{AdditionalInfos, Trace};
use crate::interface::{layer::Layer, types::Direction};

use super::FileParser;

/// PHY channel type for resource grid visualization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PHYChannelType {
    /// Physical Downlink Shared Channel
    PDSCH,
    /// Physical Uplink Shared Channel
    PUSCH,
    /// Physical Uplink Control Channel
    PUCCH,
    /// Physical Downlink Control Channel
    PDCCH,
    /// Physical Random Access Channel
    PRACH,
    /// Other PHY channel (not displayed in resource grid)
    Other,
}

/// Channel-specific data extracted from PHY traces
#[derive(Debug, Clone)]
pub enum PHYChannelData {
    /// PDCCH scheduling info (DCI 0_1 or 1_1)
    Pdcch {
        /// DCI format (e.g. "0_1", "1_1")
        dci: String,
        /// HARQ process number
        harq_process: Option<u8>,
        /// New Data Indicator
        ndi: Option<u8>,
        /// Redundancy version index
        rv_idx: Option<u8>,
        /// HARQ feedback timing (DCI 1_1 only)
        harq_feedback_timing: Option<u8>,
    },
    /// PDSCH downlink shared channel data
    Pdsch {
        /// Retransmission count
        retx: Option<u8>,
        /// Redundancy version index
        rv_idx: Option<u8>,
    },
    /// PUSCH uplink shared channel data
    Pusch {
        /// Retransmission count
        retx: Option<u8>,
        /// Redundancy version index
        rv_idx: Option<u8>,
        /// CRC result (true = OK, false = KO)
        crc: Option<bool>,
    },
    /// PUCCH uplink control channel data
    Pucch {
        /// PUCCH format (1, 2, etc.)
        format: Option<u8>,
        /// ACK/NACK (true = ACK, i.e. value != 0)
        ack: Option<bool>,
    },
    /// No channel-specific data
    None,
}

/// PHY layer information extracted from trace lines
#[derive(Debug, Clone)]
pub struct PHYInfos {
    /// Direction (UL or DL)
    pub direction: Direction,
    /// Channel type (PDSCH, PUSCH, etc.)
    pub channel_type: PHYChannelType,
    /// Frame number
    pub frame: u16,
    /// Slot number within frame
    pub slot: u8,
    /// PRB start position
    pub prb_start: u16,
    /// PRB length (number of PRBs)
    pub prb_length: u16,
    /// Symbol start position within slot
    pub symb_start: u8,
    /// Symbol length (number of symbols)
    pub symb_length: u8,
    /// HARQ process number (0-15 typically)
    pub harq: Option<u8>,
    /// True if harq=si (MIB/SIB carry, not a real HARQ process)
    pub harq_si: bool,
    /// Channel-specific parsed data
    pub channel_data: PHYChannelData,
}
    
/// PHY layer parser
pub struct PHYParser;

impl PHYParser {
    /// Parse PHY layer traces
    fn parse_lines(lines: &[String]) -> Result<Vec<String>, ParsingError> {
        Ok(lines.to_vec())
    }
}

impl FileParser for PHYParser {
    fn parse_additional_infos(lines: &[String]) -> Result<AdditionalInfos, ParsingError> {
        if lines.is_empty() {
            return Ok(AdditionalInfos::None);
        }
        
        let first_line = &lines[0];
        
        // Determine direction from the line
        let direction = if first_line.contains(" DL ") {
            Direction::DL
        } else if first_line.contains(" UL ") {
            Direction::UL
        } else {
            Direction::DL // Default to DL
        };
        
        // Try to parse PHY info (pass all lines for multi-line PDCCH parsing)
        match parse_phy_lines(lines, direction) {
            Some(phy_infos) => {
                // Store info for PDCCH/PDSCH/PUSCH/PUCCH/PRACH channels
                if matches!(phy_infos.channel_type,
                    PHYChannelType::PDCCH | PHYChannelType::PDSCH |
                    PHYChannelType::PUSCH | PHYChannelType::PUCCH |
                    PHYChannelType::PRACH) {
                    Ok(AdditionalInfos::PHYInfos(phy_infos))
                } else {
                    Ok(AdditionalInfos::None)
                }
            }
            None => Ok(AdditionalInfos::None),
        }
    }

    fn parse(lines: &[String]) -> Result<Trace, ParsingError> {
        let additional_infos = match Self::parse_additional_infos(lines) {
            Ok(m) => m,
            Err(e) => {
                return Err(e);
            }
        };
        
        let text = match Self::parse_lines(lines) {
            Ok(t) => t,
            Err(e) => {
                return Err(e);
            }
        };
        
        let binary = extract_binary_from_lines(lines);
        
        // Parse timestamp from first line
        let timestamp = if let Some(first_line) = lines.first() {
            super::parse_timestamp(first_line)?
        } else {
            0
        };
        
        let trace = Trace {
            timestamp,
            layer: Layer::PHY,
            additional_infos,
            text: Some(text),
            binary,
            relation: TraceRelation::default(),
        };
        Ok(trace)
    }
}

/// Parse a PHY layer trace line and extract RB & HARQ information
/// Accepts all lines of a PHY trace (first line + optional indented continuation
/// lines for PDCCH). Extracts frame/slot, PRB, symbol, HARQ, and channel-specific data.
///
/// Example trace formats:
/// - 5G PDSCH: `10:32:34.715 [PHY] DL 0001 01 4601  431.16 PDSCH: harq=0 prb=50 symb=1:13 k1=12...`
/// - 4G PDSCH: `10:37:50.654 [PHY] DL 0001 01 003d   421.0 PDSCH: harq=0 k1=4 prb=23:2...` (no symb)
///
/// # Arguments
/// * `lines` - All lines of the PHY trace (first line + indented fields)
/// * `direction` - Direction (UL/DL) parsed from the trace header
///
/// # Returns
/// * `Some(PHYInfos)` for handled channels (PDCCH, PDSCH, PUSCH, PUCCH, PRACH)
/// * `None` for other PHY channels or if parsing fails
pub fn parse_phy_lines(lines: &[String], direction: Direction) -> Option<PHYInfos> {
    if lines.is_empty() {
        return None;
    }
    let first_line = &lines[0];

    // Detect channel type from first line
    let channel_type = if first_line.contains("PDSCH:") {
        PHYChannelType::PDSCH
    } else if first_line.contains("PUSCH:") {
        PHYChannelType::PUSCH
    } else if first_line.contains("PUCCH:") {
        PHYChannelType::PUCCH
    } else if first_line.contains("PDCCH:") {
        PHYChannelType::PDCCH
    } else if first_line.contains("PRACH:") {
        PHYChannelType::PRACH
    } else {
        PHYChannelType::Other
    };

    // Parse frame and slot from the "frame.slot" field (e.g., "421.0")
    let (frame, slot) = parse_frame_slot(first_line)?;

    // Parse prb field — PDCCH has no prb= on first line
    let (prb_start, prb_length) = if matches!(channel_type, PHYChannelType::PDCCH) {
        (0, 0)
    } else {
        parse_prb(first_line)?
    };

    // Parse symb field — if missing, assume full slot (0:14)
    let (symb_start, symb_length) = parse_symb(first_line).unwrap_or((0, 14));

    // Parse harq field: "harq=2" -> (Some(2), false), "harq=si" -> (None, true)
    let (harq, harq_si) = parse_harq(first_line);

    // Parse channel-specific data
    let channel_data = match channel_type {
        PHYChannelType::PDCCH => parse_pdcch_data(lines),
        PHYChannelType::PDSCH => parse_pdsch_data(first_line),
        PHYChannelType::PUSCH => parse_pusch_data(first_line),
        PHYChannelType::PUCCH => parse_pucch_data(first_line),
        _ => PHYChannelData::None,
    };

    Some(PHYInfos {
        direction,
        channel_type,
        frame,
        slot,
        prb_start,
        prb_length,
        symb_start,
        symb_length,
        harq,
        harq_si,
        channel_data,
    })
}

/// Convenience wrapper for single-line parsing (used in tests)
pub fn parse_phy_line(line: &str, direction: Direction) -> Option<PHYInfos> {
    parse_phy_lines(&[line.to_string()], direction)
}

/// Parse the frame.slot field from the trace line
/// 
/// The frame.slot appears after the UE ID fields, e.g.:
/// `[PHY] DL 0001 01 003d   421.0 PDSCH:`
///                                 ^^^^^
fn parse_frame_slot(line: &str) -> Option<(u16, u8)> {
    // Look for a pattern like "  421.0 " or "  431.16 "
    // The frame.slot is typically after the layer/direction/UE fields
    
    // Split by whitespace and look for a decimal number that looks like frame.slot
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    for part in &parts {
        // Check if part contains a dot and both parts are numeric
        if let Some(dot_pos) = part.find('.') {
            let frame_part = &part[..dot_pos];
            let slot_part = &part[dot_pos + 1..];
            
            // Make sure there's no trailing punctuation on slot_part
            let slot_part: String = slot_part.chars().take_while(|c| c.is_ascii_digit()).collect();
            
            if let (Ok(frame), Ok(slot)) = (frame_part.parse::<u16>(), slot_part.parse::<u8>()) {
                // Sanity check: frame < 1024 (0-1023), slot < 20
                if slot < 20 {
                    return Some((frame, slot));
                }
            }
        }
    }
    
    None
}

/// Parse the harq field from the trace line
/// 
/// Format: "harq=2" -> (Some(2), false), "harq=si" -> (None, true)
/// Returns (None, false) if the harq field is not present
fn parse_harq(line: &str) -> (Option<u8>, bool) {
    let harq_start_idx = match line.find("harq=") {
        Some(idx) => idx,
        None => return (None, false),
    };
    let harq_value_start = harq_start_idx + 5; // Skip "harq="
    
    let value_part: String = line[harq_value_start..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    
    if value_part == "si" {
        return (None, true);
    }
    
    (value_part.parse::<u8>().ok(), false)
}

/// Extract a u8 value from a "key=value" field in a line
fn extract_field_u8(line: &str, prefix: &str) -> Option<u8> {
    let start = line.find(prefix)? + prefix.len();
    let value: String = line[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    value.parse::<u8>().ok()
}

/// Extract a string value from a "key=value" field in a line (until whitespace)
fn extract_field_str(line: &str, prefix: &str) -> Option<String> {
    let start = line.find(prefix)? + prefix.len();
    let value: String = line[start..]
        .chars()
        .take_while(|c| !c.is_whitespace())
        .collect();
    if value.is_empty() { None } else { Some(value) }
}

/// Parse PDCCH channel-specific data from multi-line trace
///
/// First line: `... PDCCH: ss_id=2 cce_index=6 al=2 dci=0_1 k2=4`
/// Subsequent indented lines: `harq_process=0`, `ndi=1`, `rv_idx=0`, etc.
fn parse_pdcch_data(lines: &[String]) -> PHYChannelData {
    let first_line = &lines[0];
    let dci = extract_field_str(first_line, "dci=").unwrap_or_default();
    let is_dci_1_1 = dci == "1_1";
    
    let mut harq_process = None;
    let mut ndi = None;
    let mut rv_idx = None;
    let mut harq_feedback_timing = None;
    
    for line in lines.iter().skip(1) {
        let trimmed = line.trim();
        if trimmed.starts_with("harq_process=") {
            harq_process = trimmed[13..].split_whitespace().next()
                .and_then(|v| v.parse::<u8>().ok());
        } else if is_dci_1_1 {
            // DCI 1_1 (DL grant): ndi1, rv_idx1, harq_feedback_timing
            if trimmed.starts_with("ndi1=") {
                ndi = trimmed[5..].split_whitespace().next()
                    .and_then(|v| v.parse::<u8>().ok());
            } else if trimmed.starts_with("rv_idx1=") {
                rv_idx = trimmed[8..].split_whitespace().next()
                    .and_then(|v| v.parse::<u8>().ok());
            } else if trimmed.starts_with("harq_feedback_timing=") {
                harq_feedback_timing = trimmed[21..].split_whitespace().next()
                    .and_then(|v| v.parse::<u8>().ok());
            }
        } else {
            // DCI 0_1 (UL grant): ndi, rv_idx
            if trimmed.starts_with("ndi=") {
                ndi = trimmed[4..].split_whitespace().next()
                    .and_then(|v| v.parse::<u8>().ok());
            } else if trimmed.starts_with("rv_idx=") {
                rv_idx = trimmed[7..].split_whitespace().next()
                    .and_then(|v| v.parse::<u8>().ok());
            }
        }
    }
    
    PHYChannelData::Pdcch { dci, harq_process, ndi, rv_idx, harq_feedback_timing }
}

/// Parse PDSCH channel-specific data from the first line
fn parse_pdsch_data(line: &str) -> PHYChannelData {
    let retx = extract_field_u8(line, "retx=");
    let rv_idx = extract_field_u8(line, "rv_idx=");
    PHYChannelData::Pdsch { retx, rv_idx }
}

/// Parse PUSCH channel-specific data from the first line
fn parse_pusch_data(line: &str) -> PHYChannelData {
    let retx = extract_field_u8(line, "retx=");
    let rv_idx = extract_field_u8(line, "rv_idx=");
    let crc = extract_field_str(line, "crc=").map(|s| s != "KO");
    PHYChannelData::Pusch { retx, rv_idx, crc }
}

/// Parse PUCCH channel-specific data from the first line
///
/// `ack` is only present on format=1. Sometimes `sr` appears instead of `ack`.
/// ack != 0 means ACK (true), ack == 0 means NACK (false).
fn parse_pucch_data(line: &str) -> PHYChannelData {
    let format = extract_field_u8(line, "format=");
    // ack field: may be multi-digit (e.g. "11", "111") — treat any non-"0" as true
    let ack = extract_field_str(line, "ack=").map(|s| s != "0");
    PHYChannelData::Pucch { format, ack }
}

/// Parse the prb field from the trace line
/// 
/// Formats:
/// - "prb=50" → (50, 1)
/// - "prb=23:2" → (23, 2)
/// - "prb=2:4" → (2, 4)
fn parse_prb(line: &str) -> Option<(u16, u16)> {
    // Find "prb=" in the line
    let prb_start_idx = line.find("prb=")?;
    let prb_value_start = prb_start_idx + 4; // Skip "prb="
    
    // Extract the value part
    let value_part: String = line[prb_value_start..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ':' || *c == ' ')
        .collect();
    
    let value_trimmed = value_part.trim();
    
    if let Some(colon_pos) = value_trimmed.find(':') {
        // Format: start:length
        let start = value_trimmed[..colon_pos].parse::<u16>().ok()?;
        let length = value_trimmed[colon_pos + 1..].parse::<u16>().ok()?;
        Some((start, length))
    } else {
        // Format: single value = start:1
        let start = value_trimmed.parse::<u16>().ok()?;
        Some((start, 1))
    }
}

/// Parse the symb field from the trace line
/// 
/// Formats:
/// - "symb=0:13" → (0, 13)
/// - "symb=1:13" → (1, 13)
/// - "symb=0:14" → (0, 14)
/// 
/// Returns None if the symb field is not present
fn parse_symb(line: &str) -> Option<(u8, u8)> {
    // Find "symb=" in the line
    let symb_start_idx = line.find("symb=")?;
    let symb_value_start = symb_start_idx + 5; // Skip "symb="
    
    // Extract the value part
    let value_part: String = line[symb_value_start..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ':' || *c == ' ')
        .collect();
    
    let value_trimmed = value_part.trim();
    
    if let Some(colon_pos) = value_trimmed.find(':') {
        // Format: start:length
        let start = value_trimmed[..colon_pos].parse::<u8>().ok()?;
        let length = value_trimmed[colon_pos + 1..].parse::<u8>().ok()?;
        Some((start, length))
    } else {
        // Format: single value = start:1
        let start = value_trimmed.parse::<u8>().ok()?;
        Some((start, 1))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_5g_pdsch() {
        let line = "10:32:34.715 [PHY] DL 0001 01 4601  431.16 PDSCH: harq=0 prb=50 symb=1:13 k1=12 CW0: tb_len=133 mod=8 rv_idx=0 cr=0.94 retx=0";
        let info = parse_phy_line(line, Direction::DL).unwrap();
        
        assert_eq!(info.frame, 431);
        assert_eq!(info.slot, 16);
        assert_eq!(info.prb_start, 50);
        assert_eq!(info.prb_length, 1);
        assert_eq!(info.symb_start, 1);
        assert_eq!(info.symb_length, 13);
        assert!(matches!(info.channel_type, PHYChannelType::PDSCH));
        assert_eq!(info.harq, Some(0));
        assert!(!info.harq_si);
        assert!(matches!(info.channel_data, PHYChannelData::Pdsch { retx: Some(0), rv_idx: Some(0) }));
    }

    #[test]
    fn test_parse_4g_pdsch_no_symb() {
        let line = "10:37:50.654 [PHY] DL 0001 01 003d   421.0 PDSCH: harq=0 k1=4 prb=23:2 tx=port0 CW0: tb_len=185 mod=6 rv_idx=0 retx=0";
        let info = parse_phy_line(line, Direction::DL).unwrap();
        
        assert_eq!(info.frame, 421);
        assert_eq!(info.slot, 0);
        assert_eq!(info.prb_start, 23);
        assert_eq!(info.prb_length, 2);
        assert_eq!(info.symb_start, 0);
        assert_eq!(info.symb_length, 14); // Default for missing symb
        assert!(matches!(info.channel_type, PHYChannelType::PDSCH));
        assert_eq!(info.harq, Some(0));
    }

    #[test]
    fn test_parse_pusch() {
        let line = "10:37:38.884 [PHY] UL 0001 01 003d   267.5 PUSCH: harq=3 prb=2:4 symb=0:13 CW0: tb_len=11 mod=2 rv_idx=0 retx=0 crc=OK snr=26.2 epre=-35.6 ta=0.1";
        let info = parse_phy_line(line, Direction::UL).unwrap();
        
        assert_eq!(info.frame, 267);
        assert_eq!(info.slot, 5);
        assert_eq!(info.prb_start, 2);
        assert_eq!(info.prb_length, 4);
        assert_eq!(info.symb_start, 0);
        assert_eq!(info.symb_length, 13);
        assert!(matches!(info.channel_type, PHYChannelType::PUSCH));
        assert_eq!(info.harq, Some(3));
        assert!(matches!(info.channel_data, PHYChannelData::Pusch { retx: Some(0), rv_idx: Some(0), crc: Some(true) }));
    }

    #[test]
    fn test_parse_pusch_crc_ko() {
        let line = "10:32:34.569 [PHY] UL 0001 01 4601  416.19 PUSCH: harq=0 prb=2 symb=0:14 CW0: tb_len=145 mod=8 rv_idx=0 cr=0.94 retx=0 crc=KO snr=37.1 epre=-86.9 ta=-0.5";
        let info = parse_phy_line(line, Direction::UL).unwrap();
        assert!(matches!(info.channel_data, PHYChannelData::Pusch { crc: Some(false), .. }));
    }

    #[test]
    fn test_parse_prb_single() {
        let line = "10:37:50.654 [PHY] DL 0001 01 003d   421.0 PDSCH: prb=50 symb=1:13";
        let info = parse_phy_line(line, Direction::DL).unwrap();
        assert_eq!(info.prb_start, 50);
        assert_eq!(info.prb_length, 1);
    }

    #[test]
    fn test_parse_pdcch_dci_0_1() {
        let lines: Vec<String> = vec![
            "10:32:29.584 [PHY] DL 0001 01 4601  942.15 PDCCH: ss_id=2 cce_index=6 al=2 dci=0_1 k2=4".into(),
            "\t\trb_alloc=0x30".into(),
            "\t\ttime_domain_rsc=1".into(),
            "\t\tmcs=27".into(),
            "\t\tndi=1".into(),
            "\t\trv_idx=0".into(),
            "\t\tharq_process=0".into(),
            "\t\tdai=3".into(),
            "\t\ttpc_command=1".into(),
            "\t\tantenna_ports=0".into(),
            "\t\tsrs_request=0".into(),
            "\t\tdmrs_seq_init=0".into(),
            "\t\tul_sch_indicator=1".into(),
        ];
        let info = parse_phy_lines(&lines, Direction::DL).unwrap();
        assert!(matches!(info.channel_type, PHYChannelType::PDCCH));
        assert_eq!(info.frame, 942);
        assert_eq!(info.slot, 15);
        assert_eq!(info.prb_start, 0); // PDCCH has no prb on first line
        match &info.channel_data {
            PHYChannelData::Pdcch { dci, harq_process, ndi, rv_idx, harq_feedback_timing } => {
                assert_eq!(dci, "0_1");
                assert_eq!(*harq_process, Some(0));
                assert_eq!(*ndi, Some(1));
                assert_eq!(*rv_idx, Some(0));
                assert_eq!(*harq_feedback_timing, None);
            }
            _ => panic!("Expected Pdcch channel data"),
        }
    }

    #[test]
    fn test_parse_pdcch_dci_1_1() {
        let lines: Vec<String> = vec![
            "10:32:22.293 [PHY] DL 0001 01 4601  213.13 PDCCH: ss_id=2 cce_index=4 al=2 dci=1_1".into(),
            "\t\trb_alloc=0x32".into(),
            "\t\tmcs1=27".into(),
            "\t\tndi1=0".into(),
            "\t\trv_idx1=0".into(),
            "\t\tharq_process=0".into(),
            "\t\tharq_feedback_timing=2".into(),
        ];
        let info = parse_phy_lines(&lines, Direction::DL).unwrap();
        match &info.channel_data {
            PHYChannelData::Pdcch { dci, harq_process, ndi, rv_idx, harq_feedback_timing } => {
                assert_eq!(dci, "1_1");
                assert_eq!(*harq_process, Some(0));
                assert_eq!(*ndi, Some(0));
                assert_eq!(*rv_idx, Some(0));
                assert_eq!(*harq_feedback_timing, Some(2));
            }
            _ => panic!("Expected Pdcch channel data"),
        }
    }

    #[test]
    fn test_parse_pucch_ack() {
        let line = "10:32:22.299 [PHY] UL 0001 01 4601  213.19 PUCCH: format=1 prb=50 prb2=0 symb=0:14 cs=1 occ=0 ack=1 snr=35.7 epre=-88.5";
        let info = parse_phy_line(line, Direction::UL).unwrap();
        assert!(matches!(info.channel_type, PHYChannelType::PUCCH));
        match &info.channel_data {
            PHYChannelData::Pucch { format, ack } => {
                assert_eq!(*format, Some(1));
                assert_eq!(*ack, Some(true));
            }
            _ => panic!("Expected Pucch channel data"),
        }
    }

    #[test]
    fn test_parse_pucch_sr() {
        let line = "13:20:47.884 [PHY] UL 003d 01 4644   960.8 PUCCH: format=1 prb=50 prb2=0 symb=0:14 cs=9 occ=2 sr=1 snr=18.9 epre=-53.8";
        let info = parse_phy_line(line, Direction::UL).unwrap();
        match &info.channel_data {
            PHYChannelData::Pucch { format, ack } => {
                assert_eq!(*format, Some(1));
                assert_eq!(*ack, None); // sr present, not ack
            }
            _ => panic!("Expected Pucch channel data"),
        }
    }

    #[test]
    fn test_parse_harq_si() {
        let line = "13:20:47.877 [PHY] DL    - 01 ffff   960.0 PDSCH: harq=si prb=41:7 symb=2:12 CW0: tb_len=84 mod=2 rv_idx=0 cr=0.44";
        let info = parse_phy_line(line, Direction::DL).unwrap();
        assert_eq!(info.harq, None);
        assert!(info.harq_si);
        assert!(matches!(info.channel_type, PHYChannelType::PDSCH));
    }

    #[test]
    fn test_parse_pucch_format2_no_ack() {
        let line = "13:20:47.885 [PHY] UL 003d 01 4644   960.9 PUCCH: format=2 prb=1 prb2=49 symb=8:2 csi=0101 epre=-51.3";
        let info = parse_phy_line(line, Direction::UL).unwrap();
        match &info.channel_data {
            PHYChannelData::Pucch { format, ack } => {
                assert_eq!(*format, Some(2));
                assert_eq!(*ack, None); // format=2 has no ack
            }
            _ => panic!("Expected Pucch channel data"),
        }
    }
}
