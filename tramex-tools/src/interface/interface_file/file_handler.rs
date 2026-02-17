//! File Handler

use crate::data::{AdditionalInfos, Data, Trace};
use crate::errors::ErrorCode;
use crate::errors::TramexError;
use crate::interface::parse_config::{FileMetadata, Technology};
use crate::interface::interface_types::InterfaceTrait;
use crate::interface::layer::Layers;
use crate::tramex_error;
use std::path::PathBuf;
use std::collections::HashMap;

use super::utils_file::parse_one_block;
use super::file_index::{FileIndex};

/// The default number of log processed by batch
const BATCH_SIZE: usize = 100;

#[derive(Debug, Clone)]
/// Data structure to store the file.
pub struct File {
    /// Path of the file.
    pub file_path: PathBuf,

    /// Content of the file.
    pub file_content: Vec<String>,

    /// Full read status of the file.
    pub full_read: bool,

    /// the number of log to read each batch
    nb_read: usize,

    /// The previous line number (deprecated - kept for compatibility)
    index_line: usize,

    /// Available
    pub available: bool,
    
    /// File index for efficient navigation (Option 3)
    pub index: Option<FileIndex>,
    
    /// Cache of parsed traces (index -> Trace)
    parsed_cache: HashMap<usize, Trace>,
    
    /// Current logical index in the file index
    pub current_log_index: usize,
}

impl Default for File {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from(""),
            file_content: vec![],
            full_read: false,
            nb_read: BATCH_SIZE,
            index_line: 0,
            available: true,
            index: None,
            parsed_cache: HashMap::new(),
            current_log_index: 0,
        }
    }
}

impl InterfaceTrait for File {
    fn get_more_data(&mut self, _layer_list: Layers, data: &mut Data) -> Result<(), Vec<TramexError>> {
        if self.full_read {
            return Ok(());
        }
        
        // Parse metadata on first call and build index
        if self.index.is_none() && data.events.is_empty() {
            data.metadata = FileMetadata::parse_from_lines(&self.file_content);
            
            // Build file index (Option 3)
            log::info!("Building file index...");
            match FileIndex::build_from_lines(&self.file_content) {
                Ok(index) => {
                    log::info!("File index built: {} logs found", index.total_count);
                    self.index = Some(index);
                }
                Err(e) => {
                    log::error!("Failed to build file index: {}", e.message);
                    return Err(vec![e]);
                }
            }
        }
        
        // Use old batch processing for now (will be optimized later)
        let (mut traces, err_processed) = self.process();
        
        // Infer technology from RRC canal name if metadata is Unknown
        if data.metadata.technology == Technology::Unknown {
            for trace in &traces {
                if let AdditionalInfos::RRCInfos(infos) = &trace.additional_infos {
                    if infos.canal.ends_with("-NR") {
                        data.metadata.technology = Technology::NR;
                    } else {
                        data.metadata.technology = Technology::LTE;
                    }
                    break; // Only need to check the first RRC trace
                }
            }
        }
        
        data.events.append(&mut traces);
        if !err_processed.is_empty() {
            let filtered = err_processed
                .iter()
                .filter(|tmx_err| !matches!(tmx_err.get_code(), ErrorCode::EndOfFile))
                .filter(|tmx_err| !matches!(tmx_err.get_code(), ErrorCode::ParsingLayerNotImplemented))
                .cloned()
                .collect();
            return Err(filtered);
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), TramexError> {
        Ok(())
    }
    
    fn supports_preloading(&self) -> bool {
        true
    }
    
    fn get_total_event_count(&self) -> Option<usize> {
        self.index.as_ref().map(|idx| idx.total_count)
    }
    
    fn is_fully_read(&self) -> bool {
        self.full_read
    }
}

impl File {
    /// Create a new file.
    pub fn new(file_path: PathBuf, file_content: String) -> Self {
        Self {
            file_path,
            file_content: file_content.lines().map(|x| x.to_string()).collect(),
            full_read: false,
            nb_read: BATCH_SIZE,
            index_line: 0,
            available: true,
            index: None,
            parsed_cache: HashMap::new(),
            current_log_index: 0,
        }
    }

    /// set file mode using a path and content
    pub fn new_file_content(file_path: PathBuf, file_content: String) -> Self {
        Self {
            file_path,
            file_content: file_content.lines().map(|x| x.to_string()).collect(),
            full_read: false,
            nb_read: BATCH_SIZE,
            index_line: 0,
            available: true,
            index: None,
            parsed_cache: HashMap::new(),
            current_log_index: 0,
        }
    }

    /// To update the number of log to read per batch
    pub fn change_nb_read(&mut self, toread: usize) {
        self.nb_read = toread;
    }

    /// To process the file and parse a batch of log
    pub fn process(&mut self) -> (Vec<Trace>, Vec<TramexError>) {
        let (vec_trace, opt_err) = File::process_string(&self.file_content, self.nb_read, &mut self.index_line);
        for one_error in &opt_err {
            if matches!(one_error.get_code(), ErrorCode::EndOfFile) {
                self.full_read = true;
            }
        }
        (vec_trace, opt_err)
    }
    /// To process a string passed in argument, with index and batch to read
    pub fn process_string(lines: &[String], nb_to_read: usize, ix: &mut usize) -> (Vec<Trace>, Vec<TramexError>) {
        let mut traces = vec![];
        let mut errors = vec![];
        for _ in 0..nb_to_read {
            if *ix >= lines.len() {
                errors.push(tramex_error!("End of file".to_string(), ErrorCode::EndOfFile));
                break;
            }
            match parse_one_block(&lines[*ix..], ix) {
                Ok(trace) => {
                    traces.push(trace);
                }
                Err(err) => {
                    log::error!("{}", err.message);
                    errors.push(err);
                }
            };
        }
        (traces, errors)
    }
}
