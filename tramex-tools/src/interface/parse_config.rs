//! Connection type and file metadata parsing

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Technology enum
pub enum Technology {
    /// LTE (4G) connection
    LTE,
    /// NR (5G) connection
    NR,
    /// Unknown connection type
    Unknown,
}

impl fmt::Display for Technology {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Technology::LTE => write!(f, "LTE (4G)"),
            Technology::NR => write!(f, "NR (5G)"),
            Technology::Unknown => write!(f, "--"),
        }
    }
}

impl Default for Technology {
    fn default() -> Self {
        Technology::Unknown
    }
}

#[derive(Debug, Clone, Default)]
/// File metadata extracted from header comments
pub struct FileMetadata {
    /// Technology (LTE or NR)
    pub technology: Technology,
    
    /// Cell information (raw line)
    pub cell_info: Option<String>,
    
    /// Version information
    pub version: Option<String>,
    
    /// Started timestamp
    pub started_on: Option<String>,

    /// Rotated timestamp
    pub rotated_on: Option<String>,
}

impl FileMetadata {
    /// Parse file metadata from header lines (lines starting with #)
    pub fn parse_from_lines(lines: &[String]) -> Self {
        let mut metadata = FileMetadata::default();
        
        for line in lines {
            let trimmed = line.trim();
            
            // Stop parsing when we hit a non-comment line
            if !trimmed.starts_with('#') {
                break;
            }
            
            // Parse connection type from Cell line
            if trimmed.starts_with("# Cell") {
                metadata.cell_info = Some(trimmed.to_string());
                
                if trimmed.contains("nr_arfcn") {
                    metadata.technology = Technology::NR;
                } else if trimmed.contains("earfcn") {
                    metadata.technology = Technology::LTE;
                }
            }
            
            // Parse version
            if trimmed.starts_with("# lteenb version") || trimmed.starts_with("# mme version") {
                metadata.version = Some(trimmed.trim_start_matches('#').trim().to_string());
            }
            
            // Parse started timestamp
            if trimmed.starts_with("# Started on") {
                metadata.started_on = Some(trimmed.trim_start_matches("# Started on").trim().to_string());
            }

            // Parse started timestamp
            if trimmed.starts_with("# Rotated on") {
                metadata.rotated_on = Some(trimmed.trim_start_matches("# Rotated on").trim().to_string());
            }
        }
        metadata
    }
    
    /// Get a specific header value by key
    pub fn get_header_value(&self, key: &str) -> Option<String> {
        match key {
            "technology" => Some(self.technology.to_string()),
            "version" => self.version.clone(),
            "started_on" => self.started_on.clone(),
            "rotated_on" => self.rotated_on.clone(),
            "cell_info" => self.cell_info.clone(),
            _ => None,
        }
    }
}
