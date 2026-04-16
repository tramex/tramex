//! Hex dump extraction utilities

/// Extract binary data from hex dump lines
///
/// Hex dump format:
/// ```text
/// 0000:  74 81 01 70 10 40 04 06  00 00 ca 00 24 68 a0 38  t..p.@......$h.8
/// 0010:  05 21 09 a0 30 00 00 46  4c 6b 6c 61 f3 70 40 20  .!..0..FLkla.p@
/// ```
///
/// # Arguments
/// * `lines` - All lines of the trace
///
/// # Returns
/// * `Some(Vec<u8>)` - Binary data if complete hex dump found
/// * `None` - If no hex dump, incomplete dump (contains "..."), or parsing error
pub fn extract_binary_from_lines(lines: &[String]) -> Option<Vec<u8>> {
    let mut binary = Vec::new();
    let mut found_hex_lines = false;
    let mut last_offset: Option<usize> = None;

    for line in lines {
        let trimmed = line.trim();

        // Check for incomplete marker (three dots, not just spaces)
        // The incomplete marker is typically "..." on its own line or "        ..."
        if trimmed == "..." || trimmed.starts_with("...") {
            return None;
        }

        // Check if line contains hex offset pattern (e.g., "0000:", "0010:")
        // Look for 4 hex digits followed by a colon
        if let Some(colon_pos) = trimmed.find(':') {
            // Extract the offset part before colon
            let before_colon = &trimmed[..colon_pos];
            let offset_str = before_colon.trim();

            // Check if it's a valid hex offset (exactly 4 hex digits)
            if offset_str.len() == 4
                && offset_str.chars().all(|c| c.is_ascii_hexdigit())
                && let Ok(offset) = usize::from_str_radix(offset_str, 16)
            {
                // Verify sequential offsets (should increment by 16)
                // But allow the first offset to be anything
                if let Some(last) = last_offset {
                    // Check if offset increments properly (typically by 16, but could vary)
                    // Allow some flexibility - just ensure it's increasing
                    if offset <= last {
                        // Non-sequential or going backwards, might be incomplete
                        return None;
                    }
                }
                last_offset = Some(offset);

                found_hex_lines = true;

                // Extract hex bytes after the colon
                let hex_part = &trimmed[colon_pos + 1..];

                // Split by whitespace and parse hex bytes
                // Stop when we hit non-hex content (ASCII representation)
                for part in hex_part.split_whitespace() {
                    // Each hex byte should be exactly 2 hex characters
                    if part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit()) {
                        if let Ok(byte) = u8::from_str_radix(part, 16) {
                            binary.push(byte);
                        }
                    } else {
                        // Hit non-hex content, stop parsing this line
                        break;
                    }
                }
            }
        }
    }

    if found_hex_lines && !binary.is_empty() {
        Some(binary)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_complete_hex() {
        let lines = vec![
            "13:21:07.077 [RRC] DL    - 01 BCCH-NR: SIB2".to_string(),
            "        0000:  74 81 01 70 10 40 04 06  00 00 ca 00 24 68 a0 38  t..p.@......$h.8".to_string(),
            "        0010:  05 21 09 a0 30 00 00 46  4c 6b 6c 61 f3 70 40 20  .!..0..FLkla.p@ ".to_string(),
        ];

        let binary = extract_binary_from_lines(&lines);
        assert!(binary.is_some(), "Binary extraction failed");
        let bytes = binary.unwrap();
        assert_eq!(bytes.len(), 32, "Expected 32 bytes, got {}", bytes.len());
        assert_eq!(bytes[0], 0x74);
        assert_eq!(bytes[1], 0x81);
        assert_eq!(bytes[16], 0x05);
    }

    #[test]
    fn test_extract_incomplete_hex() {
        let lines = vec![
            "13:20:45.574 [GTPU] TO 127.0.1.100:2152 G-PDU".to_string(),
            "        0000:  34 ff 05 b4 53 15 ca a3  00 00 00 85 01 10 01 00  4...S...........".to_string(),
            "        ...".to_string(),
        ];

        let binary = extract_binary_from_lines(&lines);
        assert!(binary.is_none());
    }

    #[test]
    fn test_no_hex_dump() {
        let lines = vec![
            "13:20:45.575 [PHY] UL 003d 01 4644   729.9 PUSCH".to_string(),
            "        Link: re@0".to_string(),
        ];

        let binary = extract_binary_from_lines(&lines);
        assert!(binary.is_none());
    }
}
