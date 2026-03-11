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
        
        // Try to parse PHY info
        match parse_phy_line(first_line, direction) {
            Some(phy_infos) => {
                // Only store info for PDSCH/PUSCH/PUCCH/PRACH channels
                if matches!(phy_infos.channel_type, PHYChannelType::PDSCH | PHYChannelType::PUSCH | PHYChannelType::PUCCH | PHYChannelType::PRACH) {
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

/// Parse a PHY layer trace line and extract RB information
///
/// Example trace formats:
/// - 5G PDSCH: `10:32:34.715 [PHY] DL 0001 01 4601  431.16 PDSCH: harq=0 prb=50 symb=1:13 k1=12...`
/// - 4G PDSCH: `10:37:50.654 [PHY] DL 0001 01 003d   421.0 PDSCH: harq=0 k1=4 prb=23:2...` (no symb)
///
/// # Arguments
/// * `line` - The first line of the PHY trace
/// * `direction` - Direction (UL/DL) parsed from the trace header
///
/// # Returns
/// * `Some(PHYInfos)` if the channel is handled (PDSCH, PUSCH, PUCCH, PRACH)
/// * `None` for other PHY channels or if parsing fails
pub fn parse_phy_line(line: &str, direction: Direction) -> Option<PHYInfos> {
    // Check if this is PDSCH or PUSCH
    let channel_type = if line.contains("PDSCH:") {
        PHYChannelType::PDSCH
    } else if line.contains("PUSCH:") {
        PHYChannelType::PUSCH
    } else if line.contains("PUCCH:") {
        PHYChannelType::PUCCH
    } else if line.contains("PDCCH:") {
        PHYChannelType::PDCCH
    } else if line.contains("PRACH:") {
        PHYChannelType::PRACH
    } else {
        // Not a channel we visualize in resource grid
        return None;
    };

    // Parse frame and slot from the "frame.slot" field (e.g., "421.0")
    let (frame, slot) = parse_frame_slot(line)?;

    // Parse prb field: "prb=50" or "prb=23:2"
    let (prb_start, prb_length) = parse_prb(line)?;

    // Parse symb field: "symb=0:13" - if missing, assume full slot (0:14)
    let (symb_start, symb_length) = parse_symb(line).unwrap_or((0, 14));

    // Parse harq field: "harq=2" -> Some(2)
    let harq = parse_harq(line);

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
    })
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
/// Format: "harq=2" -> Some(2)
/// Returns None if the harq field is not present
fn parse_harq(line: &str) -> Option<u8> {
    let harq_start_idx = line.find("harq=")?;
    let harq_value_start = harq_start_idx + 5; // Skip "harq="
    
    let value_part: String = line[harq_value_start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    
    value_part.parse::<u8>().ok()
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
        assert_eq!(info.slot, 16); // .16 in log, but our parsing will give 16
        assert_eq!(info.prb_start, 50);
        assert_eq!(info.prb_length, 1);
        assert_eq!(info.symb_start, 1);
        assert_eq!(info.symb_length, 13);
        assert!(matches!(info.channel_type, PHYChannelType::PDSCH));
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
    }

    #[test]
    fn test_parse_prb_single() {
        let line = "10:37:50.654 [PHY] DL 0001 01 003d   421.0 PDSCH: prb=50 symb=1:13";
        let info = parse_phy_line(line, Direction::DL).unwrap();
        assert_eq!(info.prb_start, 50);
        assert_eq!(info.prb_length, 1);
    }

    #[test]
    fn test_non_pdsch_pusch() {
        let line = "10:37:38.873 [PHY] UL    - 01    -   266.4 PRACH: sequence_index=9 ta=21 prb=2:6 snr=28.5";
        assert!(parse_phy_line(line, Direction::UL).is_none());
    }
}
