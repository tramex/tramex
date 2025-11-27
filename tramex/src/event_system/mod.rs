//! Event system for managing event flow and notifications

pub mod event_bus;
pub mod event_store;
pub mod data_source;
pub mod file_source;
#[cfg(feature = "websocket")]
pub mod websocket_source;
pub mod application;
pub mod integration;
pub mod example;

pub use event_bus::{EventBus, EventSubscriber, EventContext};
pub use event_store::EventStore;
pub use data_source::{DataSource, DataSourceType, FileLoadingStrategy};
pub use file_source::FileSource;
#[cfg(feature = "websocket")]
pub use websocket_source::WebSocketSource;
pub use application::Application;
pub use integration::{EventSystemBridge, create_application_with_panels, should_use_new_system};
