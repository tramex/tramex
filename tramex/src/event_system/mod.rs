//! Event system for managing event flow and notifications

pub mod application;
pub mod data_source;
pub mod event_bus;
pub mod event_store;
pub mod file_source;
pub mod integration;
#[cfg(feature = "websocket")]
pub mod websocket_source;

pub use application::Application;
pub use data_source::{DataSource, DataSourceType, FileLoadingStrategy};
pub use event_bus::{EventBus, EventContext, EventSubscriber};
pub use event_store::EventStore;
pub use file_source::FileSource;
pub use integration::create_application_with_panels;
#[cfg(feature = "websocket")]
pub use websocket_source::WebSocketSource;
