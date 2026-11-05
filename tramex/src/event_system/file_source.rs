//! File data source implementation

use super::data_source::{DataSource, DataSourceType};
use std::any::Any;
use std::path::PathBuf;
use tramex_tools::data::Data;
use tramex_tools::data::Trace;
use tramex_tools::errors::TramexError;
use tramex_tools::interface::interface_file::file_handler::File;
use tramex_tools::interface::interface_types::InterfaceTrait;
use tramex_tools::interface::layer::Layers;
use tramex_tools::interface::parse_config::FileMetadata;

/// File data source — wraps a loaded `File` and produces `Trace` batches
pub struct FileSource {
    /// File handler for reading
    handler: File,
    /// Path to the file
    path: PathBuf,
    /// Whether file is fully loaded
    fully_loaded: bool,
    /// Temporary data holder for parsing (needed by InterfaceTrait API)
    temp_data: Data,
    /// Events loaded by request_more(), returned by poll()
    pending_events: Vec<Trace>,
    /// File metadata extracted from the file header
    metadata: FileMetadata,
    /// Number of events loaded so far
    loaded_count: usize,
}

impl FileSource {
    /// Create a file source from an already-loaded `File`
    pub fn from_file(file: File) -> Self {
        let path = file.file_path.clone();
        log::info!("FileSource: Created from file {:?}", path);
        Self {
            handler: file,
            path,
            fully_loaded: false,
            temp_data: Data::default(),
            pending_events: Vec::new(),
            metadata: FileMetadata::default(),
            loaded_count: 0,
        }
    }

    /// Get the metadata extracted from the file
    pub fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }

    /// Get file handler reference
    pub fn handler(&self) -> &File {
        &self.handler
    }
}

impl DataSource for FileSource {
    fn poll(&mut self) -> Result<Vec<Trace>, Vec<TramexError>> {
        // Return any events loaded by request_more()
        let count = self.pending_events.len();
        if count > 0 {
            log::debug!("FileSource: poll returning {} events", count);
        }
        Ok(std::mem::take(&mut self.pending_events))
    }

    fn request_more(&mut self, _layers: &Layers) -> Result<(), Vec<TramexError>> {
        if self.fully_loaded {
            log::debug!("FileSource: request_more called but file already fully loaded");
            return Ok(());
        }

        log::debug!(
            "FileSource: request_more - before: loaded_count={}, pending={}",
            self.loaded_count,
            self.pending_events.len()
        );
        self.temp_data.events.clear();
        // Capture the result instead of bailing out: get_more_data appends the
        // successfully parsed traces to temp_data.events even when it also
        // returns errors, so the bookkeeping below must run regardless.
        let result = self.handler.get_more_data(Layers::default(), &mut self.temp_data);

        // Extract metadata on first batch
        if self.loaded_count == 0 {
            self.metadata = self.temp_data.metadata.clone();
            log::info!("FileSource: Extracted metadata - Technology: {:?}", self.metadata.technology);
        }

        // Update technology inference from metadata (File.get_more_data may update it)
        if self.temp_data.metadata.technology != self.metadata.technology {
            self.metadata.technology = self.temp_data.metadata.technology;
        }

        let batch_size = self.temp_data.events.len();
        self.loaded_count += batch_size;
        self.pending_events.append(&mut self.temp_data.events);
        log::debug!(
            "FileSource: request_more - loaded batch of {} events, total loaded: {}, pending: {}",
            batch_size,
            self.loaded_count,
            self.pending_events.len()
        );

        if self.handler.full_read {
            self.fully_loaded = true;
            log::info!("FileSource: File fully loaded ({} events total)", self.loaded_count);
        }

        result
    }

    fn is_auto_loading(&self) -> bool {
        false
    }

    fn toggle_auto_loading(&mut self) {
        // No-op for files
    }

    fn has_more(&self) -> bool {
        !self.fully_loaded
    }

    fn progress(&self) -> Option<f32> {
        self.handler.get_total_event_count().map(|total| {
            if total == 0 {
                1.0
            } else {
                self.loaded_count as f32 / total as f32
            }
        })
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::File {
            path: self.path.clone(),
            fully_loaded: self.fully_loaded,
        }
    }

    fn total_count(&self) -> Option<usize> {
        self.handler.get_total_event_count()
    }

    fn metadata(&self) -> Option<&FileMetadata> {
        Some(&self.metadata)
    }

    fn apply_parameters(&mut self, params: &crate::parameters::Parameters) {
        self.handler.change_nb_read(params.file_batch_size);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::missing_panics_doc)]
    use super::*;

    #[test]
    fn request_more_keeps_parsed_events_when_a_block_fails() {
        // Middle block is malformed: "DCCH-NR:" has an empty message part.
        let content = "\
10:32:20.000 [RRC] DL 0001 01 CCCH-NR: RRC setup
10:32:20.001 [RRC] DL 0001 01 DCCH-NR:
10:32:20.002 [RRC] UL 0001 01 DCCH-NR: RRC setup complete
";
        let file = File::new(PathBuf::from("test.log"), content.to_string());
        let mut source = FileSource::from_file(file);

        assert!(source.request_more(&Layers::default()).is_err());
        let events = source.poll().unwrap_or_default();
        assert_eq!(events.len(), 2);
    }
}
