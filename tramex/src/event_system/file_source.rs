//! File data source implementation

use super::data_source::{DataSource, DataSourceType, FileLoadingStrategy};
use tramex_tools::data::Trace;
use tramex_tools::data::Data;
use tramex_tools::interface::layer::Layers;
use tramex_tools::interface::interface_file::file_handler::File;
use tramex_tools::interface::interface_types::InterfaceTrait;
use tramex_tools::errors::TramexError;
use std::path::PathBuf;
use std::any::Any;

/// File data source
pub struct FileSource {
    /// File handler for reading
    handler: File,
    /// Path to the file
    path: PathBuf,
    /// Loading strategy
    strategy: FileLoadingStrategy,
    /// Number of events to load per batch
    batch_size: usize,
    /// Whether file is fully loaded
    fully_loaded: bool,
    /// Temporary data holder for parsing
    temp_data: Data,
}

impl FileSource {
    /// Create a new file source with given strategy
    pub fn new(path: PathBuf, strategy: FileLoadingStrategy) -> Result<Self, Vec<TramexError>> {
        log::info!("FileSource: Opening file {:?} with strategy {:?}", path, strategy);
        // TODO: Properly initialize File handler
        // For now, this is a placeholder
        let handler = File::default();
        
        let batch_size = match strategy {
            FileLoadingStrategy::Immediate => usize::MAX,
            FileLoadingStrategy::OnDemand { batch_size } => batch_size,
            FileLoadingStrategy::Progressive { batch_size, .. } => batch_size,
        };
        
        let mut source = Self {
            handler,
            path,
            strategy,
            batch_size,
            fully_loaded: false,
            temp_data: Data::default(),
        };
        
        // For immediate loading, load everything now
        if matches!(strategy, FileLoadingStrategy::Immediate) {
            source.load_all()?;
        }
        
        Ok(source)
    }
    
    /// Load all remaining events from file
    fn load_all(&mut self) -> Result<Vec<Trace>, Vec<TramexError>> {
        let mut all_events = Vec::new();
        
        while !self.handler.full_read {
            self.temp_data.events.clear();
            self.handler.get_more_data(Layers::default(), &mut self.temp_data)?;
            all_events.extend(self.temp_data.events.drain(..));
        }
        
        self.fully_loaded = true;
        log::info!("FileSource: Loaded all {} events", all_events.len());
        
        Ok(all_events)
    }
    
    /// Load next batch of events
    pub fn load_batch(&mut self) -> Result<Vec<Trace>, Vec<TramexError>> {
        if self.fully_loaded {
            return Ok(Vec::new());
        }
        
        self.temp_data.events.clear();
        
        // Load up to batch_size events
        let mut loaded = 0;
        while loaded < self.batch_size && !self.handler.full_read {
            let before = self.temp_data.events.len();
            self.handler.get_more_data(Layers::default(), &mut self.temp_data)?;
            let after = self.temp_data.events.len();
            loaded += after - before;
            
            // If we couldn't load any more, we're done
            if after == before {
                break;
            }
        }
        
        if self.handler.full_read {
            self.fully_loaded = true;
            log::info!("FileSource: File fully loaded ({} events total)", self.handler.get_total_event_count().unwrap_or(0));
        }
        
        Ok(self.temp_data.events.clone())
    }
    
    /// Get file handler
    pub fn handler(&self) -> &File {
        &self.handler
    }
    
    /// Get mutable file handler
    pub fn handler_mut(&mut self) -> &mut File {
        &mut self.handler
    }
}

impl DataSource for FileSource {
    fn poll(&mut self) -> Result<Vec<Trace>, Vec<TramexError>> {
        // For file sources, poll only returns data for progressive loading
        // On-demand loading happens via request_more()
        match self.strategy {
            FileLoadingStrategy::Progressive { .. } => {
                // TODO: In a full implementation, this would check a background queue
                Ok(Vec::new())
            }
            _ => Ok(Vec::new()),
        }
    }
    
    fn request_more(&mut self, _layers: &Layers) -> Result<(), Vec<TramexError>> {
        // For on-demand loading, this would trigger the next batch
        // The actual loading happens in load_batch() called from Application
        Ok(())
    }
    
    fn is_auto_loading(&self) -> bool {
        // Files don't auto-load by default (except progressive)
        matches!(self.strategy, FileLoadingStrategy::Progressive { .. })
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
                // We need to track how many events we've loaded
                // For now, use fully_loaded flag
                if self.fully_loaded {
                    1.0
                } else {
                    // Estimate based on file position would be better
                    0.0
                }
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
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
