//! Data source abstraction for different input types

use std::any::Any;
use std::path::PathBuf;
use tramex_tools::data::Trace;
use tramex_tools::errors::TramexError;
use tramex_tools::interface::layer::Layers;
use tramex_tools::interface::parse_config::FileMetadata;

/// Type of data source
#[derive(Debug, Clone)]
pub enum DataSourceType {
    /// File source
    File {
        /// Path to the file
        path: PathBuf,
        /// Whether the file is fully loaded
        fully_loaded: bool,
    },
    /// WebSocket source
    WebSocket {
        /// WebSocket URL
        url: String,
        /// Connection status
        connected: bool,
    },
}

/// Strategy for loading files
#[derive(Debug, Clone, Copy)]
pub enum FileLoadingStrategy {
    /// Load entire file at once (small files)
    Immediate,
    /// Load in batches as user navigates
    ///
    /// Load batch on demand
    OnDemand {
        /// Number of events per batch
        batch_size: usize,
    },
    /// Load in background progressively
    Progressive {
        /// Number of events per batch
        batch_size: usize,
        /// Interval between batches in milliseconds
        interval_ms: u64,
    },
}

/// Trait for different data sources (File, WebSocket, etc.)
/// threaded version, enable poll on UI render loop
#[cfg(not(target_arch = "wasm32"))]
pub trait DataSource: Send {
    /// Poll for new events (non-blocking)
    ///
    /// - WebSocket: returns received messages
    /// - File: returns empty (unless progressive loading)
    ///
    /// # Returns
    /// Vector of new events or errors
    fn poll(&mut self) -> Result<Vec<Trace>, Vec<TramexError>>;

    /// Request more data
    ///
    /// - WebSocket: sends log_get request
    /// - File: loads next batch
    ///
    /// # Returns
    /// Result indicating success or errors
    fn request_more(&mut self, layers: &Layers) -> Result<(), Vec<TramexError>>;

    /// Check if source is in auto-loading mode
    ///
    /// # Returns
    /// True if auto-loading, false otherwise
    fn is_auto_loading(&self) -> bool;

    /// Toggle auto-loading
    fn toggle_auto_loading(&mut self);

    /// Check if source has more data available
    ///
    /// # Returns
    /// True if more data available, false otherwise
    fn has_more(&self) -> bool;

    /// Get loading progress (0.0 to 1.0), None if unknown
    ///
    /// # Returns
    /// Loading progress as a float or None
    fn progress(&self) -> Option<f32>;

    /// Get source type for UI display
    ///
    /// # Returns
    /// Source type
    fn source_type(&self) -> DataSourceType;

    /// Get total event count if known
    ///
    /// # Returns
    /// Total event count or None
    fn total_count(&self) -> Option<usize> {
        None
    }

    /// Get file metadata if available (e.g., technology type)
    fn metadata(&self) -> Option<&FileMetadata> {
        None
    }

    /// Cast to Any for downcasting (immutable)
    fn as_any(&self) -> &dyn Any;

    /// Cast to Any for downcasting (mutable)
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
/// Single threaded version
#[cfg(target_arch = "wasm32")]
pub trait DataSource {
    /// Poll for new events (non-blocking)
    fn poll(&mut self) -> Result<Vec<Trace>, Vec<TramexError>>;

    /// Request more data
    fn request_more(&mut self, layers: &Layers) -> Result<(), Vec<TramexError>>;

    /// Check if source is in auto-loading mode
    fn is_auto_loading(&self) -> bool;

    /// Toggle auto-loading
    fn toggle_auto_loading(&mut self);

    /// Check if source has more data available
    fn has_more(&self) -> bool;

    /// Get loading progress (0.0 to 1.0), None if unknown
    fn progress(&self) -> Option<f32>;

    /// Get source type for UI display
    fn source_type(&self) -> DataSourceType;

    /// Get total event count if known
    fn total_count(&self) -> Option<usize> {
        None
    }

    /// Get file metadata if available
    fn metadata(&self) -> Option<&FileMetadata> {
        None
    }

    /// Cast to Any for downcasting (immutable)
    fn as_any(&self) -> &dyn Any;

    /// Cast to Any for downcasting (mutable)
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Helper trait for downcasting
impl dyn DataSource {
    /// Downcast to concrete type (for accessing source-specific methods)
    ///
    /// # Returns
    /// Reference to the concrete type if successful
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    /// Downcast to concrete type (for accessing source-specific methods)
    ///
    /// # Returns
    /// Mutable reference to the concrete type if successful
    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}
