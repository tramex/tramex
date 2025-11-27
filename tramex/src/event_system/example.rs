//! Example usage of the event system
//!
//! This demonstrates how to use the Application controller with different data sources

use super::{Application, EventSubscriber, EventContext, FileLoadingStrategy};
use super::FileSource;
use tramex_tools::data::Trace;
use std::path::PathBuf;

/// Example: Load a file and process events
pub fn example_file_loading() -> Result<(), String> {
    // 1. Create application
    let mut app = Application::new();
    
    // 2. Register panels as subscribers
    // (In real code, these would be actual panel instances)
    // app.subscribe(Box::new(Chronograph::new()));
    // app.subscribe(Box::new(RRCStatusPanel::new()));
    
    // 3. Create a file data source
    let file_path = PathBuf::from("path/to/file.bin");
    let file_source = FileSource::new(
        file_path,
        FileLoadingStrategy::OnDemand { batch_size: 1000 }
    ).map_err(|e| format!("Failed to open file: {:?}", e))?;
    
    app.set_data_source(Box::new(file_source));
    
    // 4. Main loop
    loop {
        // Update polls for new events and processes them
        if let Err(errors) = app.update() {
            for error in errors {
                eprintln!("Error: {:?}", error);
            }
        }
        
        // Check if user navigated
        // if next_button_clicked {
        //     app.navigate_next();
        // }
        
        // Check progress
        if let Some(progress) = app.progress() {
            println!("Loading: {:.0}%", progress * 100.0);
        }
        
        // Break when done (in real app, this would be in UI loop)
        if app.is_fully_loaded() {
            break;
        }
    }
    
    Ok(())
}

/// Example: WebSocket with auto-loading (commented out - needs wakeup callback)
pub fn example_websocket_loading() -> Result<(), String> {
    // 1. Create application
    // let mut app = Application::new();
    
    // 2. Register panels
    // app.subscribe(Box::new(Chronograph::new()));
    
    // 3. Connect to WebSocket
    // TODO: This requires a wakeup callback - simplified for example
    // let ws_source = WebSocketSource::connect(
    //     "ws://137.194.194.36:9001".to_string(),
    //     || {} // wakeup callback
    // ).map_err(|e| format!("Failed to connect: {:?}", e))?;
    // app.set_data_source(Box::new(ws_source));
    
    // 4. Main loop (runs continuously for WebSocket)
    // loop {
    //     // Update polls for new messages
    //     if let Err(errors) = app.update() {
    //         for error in errors {
    //             eprintln!("Error: {:?}", error);
    //         }
    //     }
    //     
    //     // Auto-loading is enabled by default, new events will:
    //     // 1. Be added to EventStore
    //     // 2. Notify all subscribers via on_event_added()
    //     // 3. Automatically navigate to last event
    //     
    //     // User can pause/resume
    //     // if pause_button_clicked {
    //     //     app.toggle_auto_loading();
    //     // }
    //     
    //     // User can manually navigate
    //     // if next_button_clicked {
    //     //     app.navigate_next();
    //     // }
    //     
    //     println!("Events loaded: {}", app.event_count());
    //     
    //     // In real app, this would be the UI frame loop
    //     // std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    // }
    
    Err("WebSocket example not yet implemented - requires wakeup callback".to_string())
}

/// Example custom subscriber
pub struct ExampleSubscriber {
    name: String,
    event_count: usize,
}

impl ExampleSubscriber {
    /// Create a new example subscriber
    pub fn new(name: String) -> Self {
        Self {
            name,
            event_count: 0,
        }
    }
}

impl EventSubscriber for ExampleSubscriber {
    fn on_event_added(&mut self, _event: &Trace, index: usize, _context: &EventContext) {
        self.event_count += 1;
        println!("{}: Received event {} at index {}", self.name, self.event_count, index);
        
        // Process event here
        // e.g., extract RRC messages, update state, etc.
    }
    
    fn on_event_focused(&mut self, _event: &Trace, index: usize, _context: &EventContext) {
        println!("{}: User navigated to index {}", self.name, index);
        
        // Update UI here
        // e.g., scroll to event, highlight, etc.
    }
    
    fn on_events_cleared(&mut self) {
        println!("{}: All events cleared", self.name);
        self.event_count = 0;
        
        // Reset state here
    }
    
    fn name(&self) -> &'static str {
        // Note: In real impl, this should be a constant
        "ExampleSubscriber"
    }
    
    fn show_window(&mut self, ctx: &egui::Context, open: &mut bool) -> Result<(), tramex_tools::errors::TramexError> {
        egui::Window::new(&self.name)
            .open(open)
            .show(ctx, |ui| {
                ui.label(format!("Example Subscriber: {} events received", self.event_count));
            });
        Ok(())
    }
}

/// Example: Using custom subscriber
pub fn example_custom_subscriber() {
    let mut app = Application::new();
    
    // Register custom subscriber
    app.subscribe(Box::new(ExampleSubscriber::new("MyPanel".to_string())));
    
    // Now any events added will be sent to the subscriber
}

/// Example: Manual event processing (for testing)
pub fn example_manual_events() {
    let mut app = Application::new();
    
    // Subscribe to events
    app.subscribe(Box::new(ExampleSubscriber::new("TestPanel".to_string())));
    
    // In real code, events would come from DataSource
    // But for testing, you could manually add them:
    // let events = vec![/* create test events */];
    // app.process_new_events(events); // Note: This would need to be public
}
