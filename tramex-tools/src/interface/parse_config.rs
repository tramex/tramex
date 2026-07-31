//! Connection type and file metadata parsing

use std::fmt;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Technology enum
pub enum Technology {
    /// LTE (4G) connection
    LTE,
    /// NR (5G) connection
    NR,
    /// Unknown connection type
    #[default]
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

    /// Physical Cell ID (PCI)
    pub pci: Option<u16>,

    /// Mode (TDD or FDD)
    pub mode: Option<String>,

    /// Frequency (ARFCN - nr_arfcn or earfcn)
    pub arfcn: Option<u32>,

    /// Number of resource blocks
    pub n_rb: Option<u16>,

    /// IO mode ("SISO" if dl_mu=1, "MIMO" otherwise)
    pub io_mode: Option<String>,

    /// SSB configuration lines from header comments (multiple SSBs possible)
    pub ssb_info: Vec<String>,
}

impl FileMetadata {
    /// Parse file metadata from header lines (lines starting with #)
    pub fn parse_from_lines(lines: &[String]) -> Self {
        let mut metadata = FileMetadata::default();

        for line in lines {
            println!("Line: {}", line);
            let trimmed = line.trim();

            // Stop parsing when we hit a non-comment line
            if !trimmed.starts_with('#') {
                break;
            }

            // Parse connection type from Cell line
            if trimmed.starts_with("# Cell") {
                metadata.cell_info = Some(trimmed.to_string());

                // Parse technology
                if trimmed.contains("nr_arfcn") {
                    metadata.technology = Technology::NR;
                } else if trimmed.contains("earfcn") {
                    metadata.technology = Technology::LTE;
                }

                // Parse PCI
                if let Some(pci_value) = Self::extract_value(trimmed, "pci=") {
                    metadata.pci = pci_value.parse().ok();
                }

                // Parse mode (TDD/FDD)
                if let Some(mode_value) = Self::extract_value(trimmed, "mode=") {
                    metadata.mode = Some(mode_value.to_uppercase());
                }

                // Parse ARFCN (try nr_arfcn first, then earfcn)
                if let Some(arfcn_value) = Self::extract_value(trimmed, "nr_arfcn=") {
                    metadata.arfcn = arfcn_value.parse().ok();
                } else if let Some(arfcn_value) = Self::extract_value(trimmed, "earfcn=") {
                    metadata.arfcn = arfcn_value.parse().ok();
                }

                // Parse n_rb
                if let Some(n_rb_value) = Self::extract_value(trimmed, "n_rb_dl=") {
                    metadata.n_rb = n_rb_value.parse().ok();
                    if let Some(n_rb_ul_value) = Self::extract_value(trimmed, "n_rb_ul=") {
                        if n_rb_ul_value != n_rb_value {
                            log::error!("n_rb_dl and n_rb_ul are different: {} and {}, this is not supported n_rb_dl has been used", n_rb_value, n_rb_ul_value);
                        }
                    }
                }

                // Parse IO mode
                if let Some(dl_mu_value) = Self::extract_value(trimmed, "dl_mu=")
                    && let Some(ul_mu_value) = Self::extract_value(trimmed, "ul_mu=")
                {
                    let input = if dl_mu_value == "1" { "SI" } else { "MI" };
                    let output = if ul_mu_value == "1" { "SO" } else { "MO" };
                    metadata.io_mode = Some(format!("{}{}", input, output));
                }
            }

            // Parse SSB header lines (collect all, not just first)
            if trimmed.starts_with("# SSB:") {
                metadata.ssb_info.push(trimmed.to_string());
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
        log::debug!("{:?}", metadata);
        metadata
    }

    /// Extract a value from a key=value pair in a string
    fn extract_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
        line.find(key).map(|start| {
            let value_start = start + key.len();
            let rest = &line[value_start..];
            // Find the end of the value (space or end of string)
            let end = rest.find(' ').unwrap_or(rest.len());
            &rest[..end]
        })
    }

    /// Get a specific header value by key
    pub fn get_header_value(&self, key: &str) -> Option<String> {
        match key {
            "technology" => Some(self.technology.to_string()),
            "version" => self.version.clone(),
            "started_on" => self.started_on.clone(),
            "rotated_on" => self.rotated_on.clone(),
            "cell_info" => self.cell_info.clone(),
            "pci" => self.pci.map(|v| v.to_string()),
            "mode" => self.mode.clone(),
            "arfcn" => self.arfcn.map(|v| v.to_string()),
            "io_mode" => self.io_mode.clone(),
            "ssb_info" => {
                if self.ssb_info.is_empty() {
                    None
                } else {
                    Some(self.ssb_info.join("; "))
                }
            }
            _ => None,
        }
    }
}
