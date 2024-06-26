//! File Handler

use crate::data::Data;
use crate::data::Trace;
use crate::errors::ErrorCode;
use crate::errors::TramexError;
use crate::interface::interface_types::InterfaceTrait;
use crate::interface::layer::Layers;
use crate::tramex_error;
use std::path::PathBuf;

use super::utils_file::parse_one_block;

/// The default number of log processed by batch
const DEFAULT_NB: usize = 50;
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

    /// The previous line number
    index_line: usize,

    /// Available
    pub available: bool,
}

impl Default for File {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from(""),
            file_content: vec![],
            full_read: false,
            nb_read: DEFAULT_NB,
            index_line: 0,
            available: true,
        }
    }
}

impl InterfaceTrait for File {
    fn get_more_data(&mut self, _layer_list: Layers, data: &mut Data) -> Result<(), Vec<TramexError>> {
        if self.full_read {
            return Ok(());
        }
        let (mut traces, err_processed) = self.process();
        data.events.append(&mut traces);
        if !err_processed.is_empty() {
            let filtered = err_processed
                .iter()
                .filter(|x| !matches!(x.get_code(), ErrorCode::EndOfFile))
                .cloned()
                .collect();
            return Err(filtered);
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), TramexError> {
        Ok(())
    }
}

impl File {
    /// Create a new file.
    pub fn new(file_path: PathBuf, file_content: String) -> Self {
        Self {
            file_path,
            file_content: file_content.lines().map(|x| x.to_string()).collect(),
            full_read: false,
            nb_read: DEFAULT_NB,
            index_line: 0,
            available: true,
        }
    }

    /// set file mode using a path and content
    pub fn new_file_content(file_path: PathBuf, file_content: String) -> Self {
        Self {
            file_path,
            file_content: file_content.lines().map(|x| x.to_string()).collect(),
            full_read: false,
            nb_read: DEFAULT_NB,
            index_line: 0,
            available: true,
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
    pub fn process_string(lines: &Vec<String>, nb_to_read: usize, ix: &mut usize) -> (Vec<Trace>, Vec<TramexError>) {
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
