//! File data source implementation

use super::data_source::{DataSource, DataSourceType};
use tramex_tools::data::Trace;
use tramex_tools::data::Data;
use tramex_tools::interface::layer::Layers;
use tramex_tools::interface::parse_config::FileMetadata;
use tramex_tools::interface::interface_file::file_handler::File;
use tramex_tools::interface::interface_types::InterfaceTrait;
use tramex_tools::errors::TramexError;
use std::path::PathBuf;
use std::any::Any;

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

        log::debug!("FileSource: request_more - before: loaded_count={}, pending={}", self.loaded_count, self.pending_events.len());
        self.temp_data.events.clear();
        self.handler.get_more_data(Layers::default(), &mut self.temp_data)?;

        // Extract metadata on first batch
        if self.loaded_count == 0 {
            self.metadata = self.temp_data.metadata.clone();
            log::info!("FileSource: Extracted metadata - Technology: {:?}", self.metadata.technology);
        }

        // Update technology inference from metadata (File.get_more_data may update it)
        if self.temp_data.metadata.technology != self.metadata.technology {
            self.metadata.technology = self.temp_data.metadata.technology.clone();
        }

        let batch_size = self.temp_data.events.len();
        self.loaded_count += batch_size;
        self.pending_events.extend(self.temp_data.events.drain(..));
        log::debug!("FileSource: request_more - loaded batch of {} events, total loaded: {}, pending: {}", batch_size, self.loaded_count, self.pending_events.len());

        if self.handler.full_read {
            self.fully_loaded = true;
            log::info!("FileSource: File fully loaded ({} events total)", self.loaded_count);
        }

        Ok(())
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
