# **Development Log**

## Overview
This document tracks major improvements to Tramex, including the indexed file navigation system, automatic batch continuation, and enhanced UI with a dedicated navigation panel.

---

### 1. Indexed Pre-Scan with Lazy Parsing

##### Implementation Date
2025-10-10

### 1.0. Overview
#### 1.0.1 Problem Solved
- Slow navigation through large log files (900k+ events)
- Landing on filtered/disabled layers when navigating
- No visibility into total event count or progress

#### 1.0.1 Solution: File Index System
Implemented a lightweight index that pre-scans the file to identify all log boundaries and extract metadata without full parsing.

#### 1.0.2. Performance Improvements
- **Index building**: ~3-4x faster (8s → 2-3s for 900k traces)
- **Navigation**: Instant jumps to any event position
- **Memory**: Only parsed events are cached
- **Filtering**: Can skip disabled layers efficiently

### 1.1. Key Components

**File Index (`file_index.rs`)**
- Scans file on load to identify all log boundaries
- Extracts metadata (timestamp and layer) for each log
- Stores line ranges for efficient random access
- Provides search methods for finding enabled logs
- Enables instant total count and progress tracking

**Enhanced InterfaceTrait**
Extended with new methods:
- `supports_preloading()` - File: true, WebSocket: false
- `get_total_event_count()` - File: Some(count), WebSocket: None
- `is_fully_read()` - Check if all data is loaded

**Updated File Handler**
- FileIndex field - built on first data request
- Parse cache - HashMap for caching parsed logs
- Automatic indexing when file is opened
- Lazy parsing - only parse events when accessed

**WebSocket Compatibility**
Implemented trait methods without changing core logic:
- Server still handles parsing and filtering
- No preloading support (server controls data)
- Unknown total count (returns None)

### 1.2. `tramex-tools` Architecture

The `tramex-tools` crate provides the data acquisition and parsing layer used by Tramex. Its main entry point is `Connector`, which reads traces from supported sources and returns `Data`.

#### 1.2.1. Data Model

`Data` contains a vector of `Trace` values and the index of the current trace. Each `Trace` combines:

- `MessageType`: metadata such as the protocol layer, direction, and channel.
- `Hexa`: the hexadecimal representation of ASN.1 binary data, which can be decoded according to the message type.

#### 1.2.2. Interfaces

Tramex retrieves 4G traces from an Amarisoft software core through two interfaces:

- `File`, which reads multiple traces from a log file.
- `WsConnection`, which requests traces through the WebSocket API.

`Connector` exposes constructors for these sources and two principal data-loading methods:

- `get_more_data` requests more traces. For WebSocket sources, it sends a message to the server; for file sources, it reads the next unread part of the file and stores the resulting data.
- `try_recv` reads the WebSocket receive buffer and is typically called whenever the UI refreshes.

When a WebSocket connection is open, server messages enter a receive buffer. The application can either poll this buffer or use a callback to wake the UI when a message arrives.

#### 1.2.3. Parsing

Both interfaces serialize their input into the common trace model. File parsing starts in `file_handler`, with the indexed pre-scan and lazy parsing described above allowing traces to be located before their full payload is decoded.

### 1.3. Files Modified
**New Files:**
- `tramex-tools/src/interface/interface_file/file_index.rs`

**Modified Files:**
- `tramex-tools/src/interface/interface_types.rs`
- `tramex-tools/src/interface/interface_file/file_handler.rs`
- `tramex-tools/src/interface/interface_file/mod.rs`
- `tramex-tools/src/interface/websocket/ws_connection.rs`
- `tramex/src/handlers/mod.rs`
- `tramex/src/handlers/handler_file.rs`

---

## 2. Automatic Batch Continuation

#### Implementation Date
2025-10-10

### 2.0. Overview
#### 2.0.1. Problem Solved
When navigating with layer filters enabled, the "Next" button would sometimes stop after loading 5-6 batches without finding an enabled layer, requiring multiple clicks to continue.

#### 2.0.2. Solution: Loop-Based Batch Loading
Modified the frontend to keep loading batches in a loop until an enabled layer is found or the end of file is reached.

#### 2.0.3. Benefits
- **No more manual clicking**: One click finds the next enabled event
- **Faster navigation**: Automatic batch loading feels instant
- **Better UX**: Users don't need to understand batching

### 2.1. Architecture
1. User clicks "Next"
2. Search through currently loaded events
3. If no enabled layer found → load next batch
4. Search through new batch
5. If still not found → load another batch
6. **Repeat until**: enabled layer found OR end of file OR error

#### 2.1.1. Implementation Details
**In `frontend.rs`:**
```rust
while self.trame_manager.should_get_more_log {
    // Load batch
    // Continue navigation
    // Loop continues if more batches needed
}
```

**In `trame_manager.rs`:**
- Sets `should_get_more_log = true` when reaching end of loaded events
- Sets `continue_navigation_after_load = true` to resume search
- Automatically continues until enabled layer found

### 2.2. Files Modified
- `tramex/src/frontend.rs` - Added while loop for batch continuation
- `tramex/src/panels/trame_manager.rs` - Navigation logic with flags

---

## 3. Enhanced Navigation UI

#### Implementation Date
2025-10-10

### 3.0. Overview
#### 3.0.1. Problem Solved
- Limited visibility into navigation state
- No clear indication of progress or total events
- Plain, unstyled controls

#### 3.0.2. Solution: Dedicated Navigation Panel + Enhanced Controls

#### 3.0.3. Future Enhancements

##### 3.0.3.1 Navigation Features
1. **Jump to Event**: Input field to jump to specific event number
2. **Keyboard shortcuts**: Arrow keys for navigation
3. **Event markers**: Visual indicators for important events
4. **Timeline view**: Visual timeline of events
5. **Search**: Search through events by content
6. **Bookmarks**: Mark and save important event positions

#### 3.0.3.2 Performance
1. **Parallel indexing**: Build index in background thread
2. **Incremental updates**: Update index as file grows
3. **Smart caching**: LRU cache for parsed events
4. **Batch size optimization**: Adaptive batch sizes

#### 3.0.3.3 UI/UX
1. **Filtering UI**: Visual filter controls in navigation panel
2. **Statistics graphs**: Charts showing event distribution
3. **Performance metrics**: Show parsing speed, memory usage
4. **Batch controls**: Skip forward/backward by N events
5. **Auto-play**: Automatically advance through events
6. **Dark/Light themes**: Theme support


### 3.1. New Navigation Panel Window (`navigation_panel.rs`)

Navigation Panel Window Layout
```
┌─────────────────────────────────────────┐
│  Navigation                     [X]     │
├─────────────────────────────────────────┤
│  Event: 1 / 1234   |  Loaded: 150       │
│  Time: 12:34:56.789   |  Layer: RRC     │
│  ─────────────────────────────────────  │
│   [  ◀ Previous  ]        [  ▶ Next  ] │
└─────────────────────────────────────────┘
```

#### 3.1.1. Functionalities
A floating window that displays comprehensive navigation statistics:

**Statistics Display:**
- **Event**: Current position with total count (e.g., "1 / 1234")
- **Loaded**: Events currently in memory
- **Time**: Current event timestamp (HH:MM:SS.FFF format)
- **Layer**: Current event layer

**Navigation Controls:**
- **▶ Next** button (blue, rounded corners)
- **◀ Previous** button (brown, rounded corners)
- Centered layout for better visual balance
- Large buttons (150x40px) for easy clicking

**Features:**
- Opens by default with other windows
- Can be toggled via Windows menu
- Resizable and movable
- Real-time updates as you navigate

### 3.2. Menu Organization
Moved Windows menu to top menu bar:
- **Menu**: Organize windows, About, Quit
- **Windows**: Toggle all panels (Navigation, Message Box, etc.)
- **About**: Documentation links

### 3.3. Files Modified
**New Files:**
- `tramex/src/panels/navigation_panel.rs`

**Modified Files:**
- `tramex/src/panels/mod.rs` - Added navigation_panel module
- `tramex/src/frontend.rs` - Added NavigationPanel, updated menu integration
- `tramex/src/app.rs` - Integrated Windows menu in top bar
- `tramex/src/panels/panel_message.rs` - Re-added "Show full message" checkbox
---

## 4. RRC Field Viewer & Message Panel Improvements

#### Implementation Date
2025-10-10

### 4.0. Overview
#### 4.0.1. Problem Solved
- No dedicated panel for viewing specific RRC message fields
- Message panel displayed too much information (timestamp, hex always visible)
- Array values in JSON were not properly displayed
- Technology information was not visible in logical channels panel
- Technology detection relied only on file headers (which could be missing)

#### 4.0.2. Solution: RRC Field Viewer + UI Enhancements

#### 4.0.3. Benefits

**For Users:**
- Quick access to important RRC message fields
- Cleaner message panel with less clutter
- Technology always visible and correctly detected
- Easy to understand array values

**For Developers:**
- Easy to add new message configurations
- Flexible JSON path system
- Centralized technology detection
- Extensible field mapping system

#### 4.0.4. Future Enhancements

1. **Dynamic field configuration**: Load field mappings from config file
2. **More message types**: Add configurations for other RRC messages (SIB2, MIB, etc.)
3. **Field filtering**: Show/hide specific fields
4. **Export fields**: Copy field values to clipboard
5. **Field history**: Track field value changes over time
6. **Comparison view**: Compare fields across multiple messages

### 4.1. New RRC Field Viewer Panel (`bst_config.rs`)
A dedicated panel for displaying extracted fields from RRC messages with configurable field mappings.

**Key Features:**
- **Configurable field mappings**: Define which fields to extract per message type
- **JSON path navigation**: Support for nested objects and array indexing with `[N]` syntax
- **Automatic value formatting**: 
  - Arrays displayed as `[item1, item2, ...]`
  - Objects with `decimal`/`hex` fields automatically formatted
  - Units can be added to values (e.g., "dB")
- **Grid layout**: Clean table display with field names and values

**Default Configurations:**
- **SIB1 (4G)**: mcc, mnc (from `plmn-IdentityList[0].plmn-Identity`)
- **SIB1 (5G)**: q-RxLevMin, q-QualMin, mcc, mnc, cellId, trackingAreaCode (from `plmn-IdentityInfoList[0]`)

**JSON Path Examples:**
```rust
// Simple field
"message.c1.systemInformationBlockType1.cellSelectionInfo.q-RxLevMin"

// Array indexing
"message.c1.systemInformationBlockType1.cellAccessRelatedInfo.plmn-IdentityList[0].plmn-Identity.mcc"

// Nested arrays
"message.c1.systemInformationBlockType1.cellAccessRelatedInfo.plmn-IdentityInfoList[0].plmn-IdentityList[0].mcc"
```

**Value Extraction:**
- **Arrays**: `[0, 0, 1]` → displayed as `[0, 0, 1]`
- **Objects with decimal**: `{"decimal": 101, "hex": "000065"}` → displayed as `101`
- **Objects with hex**: `{"hex": "001234501"}` → displayed as `0x001234501`

### 4.2. Message Panel Improvements
**UI Cleanup:**
- **Layer type**: Now displayed as large heading
- **Timestamp removed**: No longer displayed by default
- **Hex moved**: Only shown in "Show full message" section for RRC layers
- **Simplified layout**: Less clutter, more focus on important info

**Before:**
```
RRC at 12:34:56.789
AdditionalInfos(...)
Hex: [0x12, 0x34, ...]
[Show full message checkbox]
```

**After:**
```
Layer: RRC

AdditionalInfos(...)
[Show full message checkbox]

[When checked:]
  Hex: [0x12, 0x34, ...]
  [ASN.1 text...]
```

### 4.3. Technology Detection Enhancement
**Two-tier detection system:**

1. **Primary**: File header parsing (existing)
   - Looks for `nr_arfcn` (5G) or `earfcn` (4G) in Cell line
   
2. **Fallback**: RRC canal name inference (new)
   - If technology is `Unknown` after header parsing
   - Checks first RRC trace canal name:
     - Ends with `"-NR"` → `Technology::NR`
     - Otherwise → `Technology::LTE`
   - Applied during file parsing in `file_handler.rs`

**Benefits:**
- Works even when file headers are missing
- Propagates to all panels automatically
- Displayed in Logical Channels panel

### 4.4. Logical Channels Panel Update
**New display:**
```
[----] [----] [----] [Downlink] [----] [----] [----]
[    ] [Techno: LTE (4G)] [    ] [----] [Uplink] [----]
```

Shows technology (LTE/NR) in the channel grid for quick reference.


### 4.5. Technical Details

**Array Indexing Implementation:**
```rust
// Supports syntax like "field[0].nested[1].value"
if let Some(bracket_pos) = part.find('[') {
    let field_name = &part[..bracket_pos];
    let index_str = &part[bracket_pos+1..part.len()-1];
    
    current = current.get(field_name)?;
    
    if let Value::Array(arr) = current {
        let index: usize = index_str.parse().ok()?;
        current = arr.get(index)?;
    }
}
```

**Technology Inference:**
```rust
// In file_handler.rs during batch processing
if data.metadata.technology == Technology::Unknown {
    for trace in &traces {
        if let AdditionalInfos::RRCInfos(infos) = &trace.additional_infos {
            if infos.canal.ends_with("-NR") {
                data.metadata.technology = Technology::NR;
            } else {
                data.metadata.technology = Technology::LTE;
            }
            break;
        }
    }
}
```

### 4.5. Files Modified

**New Files:**
- `tramex/src/panels/bst_config.rs` - New RRC field viewer panel

**Modified Files:**
- `tramex/src/panels/mod.rs` - Added bst_config module
- `tramex/src/frontend.rs` - Integrated RRC Field Viewer panel
- `tramex/src/panels/panel_message.rs` - UI improvements, moved display_log function
- `tramex/src/utils.rs` - Removed display_log (moved to panel_message.rs)
- `tramex/src/panels/logical_channels.rs` - Added technology display
- `tramex-tools/src/interface/interface_file/file_handler.rs` - Technology inference logic
- `tramex-tools/src/interface/parse_config.rs` - Technology enum and FileMetadata


---

## 5. WebSocket Connection Debugging Guide

#### Implementation Date
2025-11-27

### 5.0. Overview
#### 5.0.1. Current Situation

You're receiving the initial "ready" message from the Amarisoft server but not seeing subsequent log messages.

#### 5.0.2. Root Cause

**The Amarisoft WebSocket server uses a REQUEST-RESPONSE pattern**, not a push model:

1. ✅ Server sends "ready" message when you connect
2. ❌ **You must send `log_get` requests to receive logs** (this is not happening automatically)
3. ❌ Server only sends log data in response to your `log_get` requests


#### 5.0.3. Next Steps

1. **Run your application** and check the logs for the emoji indicators (🔵 📨 📤 ✅)
2. **Click the Next button** to trigger a `log_get` request
3. **Check if you see the request being sent** (📤 messages)
4. **Check if you receive a response** (🔵 and 📨 messages)
5. **If no response**, check the Amarisoft server configuration for screen settings

### 5.1. How Your Application Works

#### 5.1.1 Message Flow:
```
1. Connect → Server sends "ready" message
2. User navigates (clicks Next) → Triggers `should_get_more_log = true`
3. `get_more_data()` is called → Sends `log_get` request to server
4. Server responds → `try_recv()` receives the log data
```

#### 5.1.2. The Problem:
- `try_recv()` is called every frame to check for incoming messages ✅
- `get_more_data()` sends the request to the server ✅
- **BUT** `get_more_data()` is only called when:
  - You click the "Next" navigation button
  - You're near the end of loaded events (preloading)
  - `should_get_more_log` is set to `true`

### 5.2. Debugging Steps

#### 5.2.1. Check the Logs

I've added comprehensive logging to help you debug. Run your application and look for these log messages:

```
✅ WebSocket connection opened successfully
✅ Received 'ready' message from server: ENB
💡 Server is ready. You need to click 'Load More' or enable auto-loading to request logs.
```

#### 5.2.2. Trigger a Request

After connecting, you need to trigger a `log_get` request. Try one of these:

**Option A: Click the Navigation Button**
- Click the "Next" button in your navigation panel
- This should trigger `get_more_data()` and you'll see:
  ```
  📤 Sending log_get request: ...
  📤 JSON request: {"timeout":1,"min":64,"max":1024,...}
  ```

**Option B: Check if Auto-Loading Works**
- The code has preloading logic that should automatically request more data
- But it only works if you have some events already loaded

#### 5.2.3. Watch for Server Response

After sending a request, you should see:
```
🔵 WebSocket event received: Message(...)
📨 Raw WebSocket text message: {"message":"log_get","logs":[...]}
```

If you DON'T see the server response, the problem is with the server or network.

### 5.3. Common Issues

#### 5.3.1. "Screens" on Amarisoft Server

Amarisoft servers often require you to enable "screens" to receive log data:

**Check if screens are enabled:**
```bash
## Connect to your Amarisoft VM
ssh user@137.194.194.35

## Check screen configuration
## Look for screen settings in your ENB configuration file
```

**Typical screen configuration in `enb.cfg` or `mme.cfg`:**
```
log_options: {
    // Enable screens
    screens: [
        {
            name: "ENB",
            layers: ["rrc", "nas", "s1ap"],  // Specify which layers to log
        }
    ],
}
```

#### 5.3.2. Layer Filters

Your `log_get` request includes layer filters. Make sure:
1. The layers you're requesting are enabled on the server
2. The layers match what the server is configured to send

**Check your layer configuration:**
- Open the Options panel in your app
- Make sure at least one layer is enabled (e.g., RRC, NAS, S1AP)

#### 5.3.3. Server Not Sending Data

The server might not have any data to send if:
- No UE (User Equipment) is connected
- No traffic is being generated
- The layers you're requesting have no activity

### 5.4. Testing with a Simple Request

You can test the WebSocket connection manually using a WebSocket client:

```javascript
// Connect to the server
const ws = new WebSocket('ws://137.194.194.35:9001');

ws.onopen = () => {
    console.log('Connected');
    
    // Wait for ready message, then send a log_get request
    setTimeout(() => {
        const request = {
            message: "log_get",
            message_id: 1,
            timeout: 1,
            min: 64,
            max: 1024,
            layers: {
                rrc: "Debug",
                nas: "Debug",
                s1ap: "Debug"
            },
            headers: false
        };
        ws.send(JSON.stringify(request));
        console.log('Sent log_get request');
    }, 1000);
};

ws.onmessage = (event) => {
    console.log('Received:', event.data);
};
```

### 5.5. Files Modified

I've added logging to:
- `tramex-tools/src/interface/websocket/ws_connection.rs`
  - Line 115: Log all WebSocket events
  - Line 122: Log all raw text messages
  - Line 74-77: Log all outgoing requests
  - Line 148-149: Log ready message with helpful hint

All logs use `log::info!()` so they'll be visible by default.


---

## 6. Event-Driven Architecture (Observer Pattern)

#### Implementation Date
2025-11-09 : Implementation
2026-03-11 : Clean up the legacy system

### 6.0. Overview
Refactored the application to use an event-driven architecture with the Observer pattern, replacing the legacy polling-based system with a reactive, notification-based approach.

### 6.1. Architecture

**Core Components:**

1. **Application Controller** - Central orchestrator
   - Owns `EventStore` (all event data)
   - Owns `EventBus` (notification dispatcher)
   - Owns `DataSource` (File/WebSocket I/O)
   - Coordinates event flow and navigation

2. **EventStore** - Event data management
   - Single source of truth for all events
   - Maintains current index for navigation
   - Provides O(1) access to events

3. **EventBus** - Observer pattern implementation
   - Dispatches notifications to all subscribers
   - Three notification types:
     - `on_event_added` - New event received
     - `on_event_focused` - User navigated to event
     - `on_events_cleared` - Data cleared

4. **EventSubscriber Trait** - Panel interface
   - All panels implement this trait
   - Receive notifications automatically
   - No manual polling required

### 6.2. Data Flow

**Loading Events:**
```
DataSource → Application.update()
          → EventStore.add_events()
          → EventBus.notify_event_added()
          → All panels' on_event_added()
```

**Navigation:**
```
User clicks Next → Application.navigate_next()
                → EventStore.go_next()
                → EventBus.notify_event_focused()
                → All panels' on_event_focused()
```

**Auto-Loading (WebSocket):**
```
Frame loop → Application.update()
          → DataSource.poll()
          → New events processed
          → Auto-navigate to latest
```

### 6.3. Key Benefits

| Aspect | Old System | New System |
|--------|-----------|------------|
| **Event Processing** | Loop through all events on navigation | Process once when added |
| **Data Ownership** | Scattered across Data, TrameManager, panels | Single EventStore |
| **Navigation** | Coupled with processing | Separate: data vs. UI focus |
| **Adding Panels** | Manually wire in multiple places | Implement EventSubscriber trait |
| **Testing** | Hard to test individual parts | Easy to mock components |

### 6.4. API Examples

**Basic Usage:**
```rust
// Create application
let mut app = Application::new();

// Register panels
app.subscribe(Box::new(Chronograph::new()));
app.subscribe(Box::new(RRCStatusPanel::new()));

// Set data source (File or WebSocket)
app.set_data_source(Box::new(file_source));

// Main loop
loop {
    app.update()?;  // Poll data, process events, notify panels
    
    // Navigation
    if next_clicked {
        app.navigate_next();
    }
}
```

**File Loading Strategies:**
```rust
// Load entire file immediately
FileSource::new(path, FileLoadingStrategy::Immediate)

// Load in batches on demand
FileSource::new(path, FileLoadingStrategy::OnDemand { batch_size: 1000 })

// Progressive loading with delays
FileSource::new(path, FileLoadingStrategy::Progressive { 
    batch_size: 500, 
    delay_ms: 100 
})
```

### 6.5. Panel Implementation

Panels implement `EventSubscriber`:

```rust
impl EventSubscriber for Chronograph {
    fn on_event_added(&mut self, event: &Trace, index: usize, context: &EventContext) {
        // Process event data (extract info, update state)
        if let AdditionalInfos::RRCInfos(infos) = &event.additional_infos {
            self.add_arrow(infos, index);
        }
    }
    
    fn on_event_focused(&mut self, event: &Trace, index: usize, context: &EventContext) {
        // Update UI (scroll, highlight)
        self.current_index = index;
        self.should_scroll = true;
    }
    
    fn on_events_cleared(&mut self) {
        // Clean up
        self.arrows.clear();
    }
}
```

### 6.6. Performance Characteristics

**Memory:**
- Single copy of events in EventStore
- No duplication across panels
- Bounded collections (e.g., max 500 arrows in Chronograph)

**CPU:**
- O(1) event addition to store
- O(n) notification where n = number of subscribers
- O(1) navigation between events

**I/O:**
- File: Configurable batch size (1-4096 events)
- WebSocket: Server-controlled with timeout
- Non-blocking: poll() never blocks main thread

### 6.7. Files

- `tramex/src/event_system/mod.rs` - Module exports
- `tramex/src/event_system/application.rs` - Application controller
- `tramex/src/event_system/event_bus.rs` - Observer pattern dispatcher
- `tramex/src/event_system/event_store.rs` - Event data management
- `tramex/src/event_system/data_source.rs` - DataSource trait
- `tramex/src/event_system/file_source.rs` - File DataSource
- `tramex/src/event_system/websocket_source.rs` - WebSocket DataSource
- `tramex/src/event_system/integration.rs` - `create_application_with_panels()` factory
- `tramex/src/frontend.rs` - Integrated Application controller
- `tramex/src/panels/*.rs` - All panels implement EventSubscriber

### 6.8. Legacy Cleanup (2025-03-11)

Removed legacy migration scaffolding:
- `EventSystemBridge` struct and all methods
- `should_use_new_system()` env-var toggle
- `add_event()` (unused singular method)
- `TRAMEX_USE_NEW_EVENT_SYSTEM` env var from `.cargo/config.toml`
- Legacy comments throughout `application.rs` and `frontend.rs`

**Still in place** (requires `Handler` trait refactoring):
- `FrontEnd.data: Data` — handlers still write into `Data`
- `Handler → Data → add_events() → Application` transfer pipeline
- `sync_metadata_from_data()` bridge

### 6.9. Future Enhancements

1. **Complete DataSource migration**: Replace `Handler` trait with `DataSource`, remove `Data` middleman
2. **Advanced features**: Event filtering, search, bookmarks, export/import

---

## 7. Trace Association System

#### Implementation Date
2025-01-11

### 7.0. Overview
A flexible, rule-based system for linking related protocol messages across layers, enabling visibility into parent/child relationships between traces.

### 7.1. Key Principles

#### 7.1.1. Parent-Child Relationships

Each `Trace` contains a `TraceRelation` that stores its associations:

**TraceRelation** holds two `AssociationStatus` fields:
```rust
pub struct TraceRelation {
    pub parent: AssociationStatus,  // Traces that carry this one
    pub child: AssociationStatus,   // Traces carried by this one
}
```

**AssociationStatus** represents the state of an association search:
```rust
pub enum AssociationStatus {
    NotComputed,           // Not yet processed
    Found(Vec<usize>),     // Found related trace indices
    NotFound,              // Searched but no match found
    NotApplicable,         // Layer doesn't support associations
}
```

**Why `Vec<usize>` for multiple associations?**  
This is specific to **NAS**, which is carried by both RRC (radio layer) and NGAP (core network). A NAS message can have two parents simultaneously. For logic convenience and consistency, this multi-parent support has been applied to all layers, and extended to children as well for shared logic and possible evolution.

#### 7.1.2. Window-Based Search

Associations are found by searching within a **configurable window** around the source trace:

```
         ←── window_size ──→            ←── window_size ──→
    [...][candidate][candidate][SOURCE][candidate][candidate][...]
              <-                    ↑                  ->
         search_backward      current trace      search_forward
```

- **Default window**: 10 traces in each direction
- **Efficiency**: Avoids scanning entire event list. 
- **Configurable**: Each rule can override `window_size()`

#### 7.1.3. Preferred Direction

Rules specify which direction to search first based on the trace's communication direction:

```rust
pub enum SearchDirection {
    BackwardFirst,   // Search past traces first, then future
    ForwardFirst,    // Search future traces first, then past
    BackwardOnly,    // Only search past traces
    ForwardOnly,     // Only search future traces
}
```

**Typical usage:**
- **Uplink (UL/TO)**: Search backward first (carrier came before because the ENB/GNB is reading encapsulation with lower level first)
- **Downlink (DL/FROM)**: Search forward first (carrier comes after because the ENB/GNB is encapsulating higher level first)

#### 7.1.4. Message Filtering

Rules can filter which messages are eligible for association:

- **`valid_source_messages()`**: Only process source traces with these message names
- **`valid_target_messages()`**: Only consider targets with these message names

This prevents false matches between unrelated message types.

#### 7.1.5. Relationship Direction (`source_is_child`)

Each rule defines whether the source trace is the child or parent:

| `source_is_child` | Source Role | Target Role | Example |
|-------------------|-------------|-------------|---------|
| `true` | Child | Parent | NAS finds its RRC carrier |
| `false` | Parent | Child | NGAP finds the NAS it carries |

### 7.2. Architecture

```
┌─────────────────────────────────────────────────────────┐
│                 AssociationRules                        │
│         (Collection of rule implementations)            │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   TraceMatcher                          │
│  - find_relative(source, rules) → AssociationStatus    │
│  - search_backward() / search_forward()                │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   TraceRelation                         │
│  - parent: AssociationStatus (Vec<usize>)              │
│  - child: AssociationStatus (Vec<usize>)               │
└─────────────────────────────────────────────────────────┘
```

### 7.3. AssociationRule Trait

```rust
pub trait AssociationRule {
    // Layer configuration
    fn source_layer(&self) -> Layer;
    fn target_layer(&self) -> Layer;
    
    // Core matching logic
    fn matches(&self, source: &Trace, candidate: &Trace) -> bool;
    
    // Search behavior
    fn window_size(&self) -> usize { 10 }
    fn preferred_direction(&self, source: &Trace) -> SearchDirection;
    
    // Filtering
    fn valid_source_messages(&self) -> &[&str] { &[] }
    fn valid_target_messages(&self) -> &[&str] { &[] }
    
    // Relationship direction
    fn source_is_child(&self) -> bool { true }
}
```

### 7.4. Computation Flow

#### 7.4.1. compute_associations()

Main entry point that processes traces and builds relationships:

```rust
pub fn compute_associations(
    events: &mut [Trace], 
    rules: &AssociationRules, 
    start_index: usize
) {
    for index in start_index..events.len() {
        // 1. Get applicable rules for this trace's layer
        let applicable_rules = rules.rules_for_layer(&layer);
        
        // 2. If no rules apply, mark as NotApplicable
        if applicable_rules.is_empty() {
            trace.relation.set_parent_not_applicable();
            continue;
        }
        
        // 3. Try each rule until match found
        for rule in applicable_rules {
            let status = TraceMatcher::find_relative(index, events, rule);
            
            if let AssociationStatus::Found(targets) = status {
                // 4. Set bidirectional relationship based on source_is_child
                if rule.source_is_child() {
                    trace.add_parent(target);      // source is child
                    target.add_child(index);       // target is parent
                } else {
                    trace.add_child(target);       // source is parent
                    target.add_parent(index);      // target is child
                }
            }
        }
    }
}
```

#### 7.4.2. rules_for_layer()

Returns all rules where the given layer is the **source layer**:

```rust
pub fn rules_for_layer(&self, layer: &Layer) -> Vec<&dyn AssociationRule> {
    self.rules.iter()
        .filter(|r| r.source_layer() == *layer)
        .collect()
}
```

This means:
- When processing a **NAS** trace → `NasToRrcRule` applies (NAS is source, finds RRC parent)
- When processing an **NGAP** trace → `NgapToNasRule` applies (NGAP is source, finds NAS child)

#### 7.4.3. Lookback Window

When new batches arrive, traces near the batch boundary are re-processed to catch associations that span batches:

```rust
let lookback_window = 20;
let lookback_start = start_index.saturating_sub(lookback_window);
// Reset NotFound status in lookback range to NotComputed
// Then recompute from lookback_start
```

### 7.5. Chronograph Integration

Related traces are visually highlighted:
- **Current trace**: Bright blue
- **Parent/child traces**: Lighter blue
- **Other traces**: Theme-aware default color

### 7.6. Implemented Rules

#### NasToRrcRule
| Parameter | Value |
|-----------|-------|
| Source Layer | NAS |
| Target Layer | RRC |
| `source_is_child` | `true` (NAS is child) |
| Window Size | 10 |
| Matching | Binary comparison (first 10 bytes) |
| Valid Source | All NAS messages |
| Valid Target | `dl information transfer`, `ul information transfer`, `rrc setup complete`, `rrc reconfiguration` |

#### NgapToNasRule
| Parameter | Value |
|-----------|-------|
| Source Layer | NGAP |
| Target Layer | NAS |
| `source_is_child` | `false` (NGAP is parent) |
| Window Size | 10 |
| Matching | Binary comparison (first 14 bytes) |
| Valid Source | `initial ue message`, `downlink nas transport`, `uplink nas transport`, `initial context setup request` |
| Valid Target | All NAS messages |


#### Note on Rule Direction

| Rule | Direction |
|------|-----------|
| `NgapToNasRule` | `source_is_child = false` |
| `NasToRrcRule` | `source_is_child = true` |

There is no particular reason for this difference. Both orders are valid. The variation was primarily for testing the full logic.

**Performance consideration:** There is no difference in performance between the two approaches. However, `source_is_child = false` could be slightly faster if the hex parsing is reused across matches instead of being parsed every time.


### 7.7. Files

**New Files:**
- `tramex-tools/src/interface/association/mod.rs` - `compute_associations()`
- `tramex-tools/src/interface/association/relation.rs` - `TraceRelation`, `AssociationStatus`
- `tramex-tools/src/interface/association/rules.rs` - `AssociationRule` trait, implementations
- `tramex-tools/src/interface/association/matcher.rs` - `TraceMatcher`

---

## 8. Resource Blocks Panel - 5G NR Resource Grid Visualization

#### Implementation Timeline
- **v1.0**: 2025-02-11 - Initial grid visualization
- **v2.0**: 2025-02-28 - HFN support, view modes, performance optimizations

#### Overview
A visual resource grid panel displaying 5G NR Physical Resource Block (PRB) allocations over time. Renders a scrollable matrix of **PRBs (y-axis) × Time (x-axis)** with two view modes: Symbol-level (detailed) and Slot-level (aggregated).

### 8.1. Key principles

#### 8.1.1. Frequency Axis

The Y axis is the frequency axis. 
There is no specific name for the unit, it will be referenced as **Physical Resource Blocks (PRB)**, although one PRB is a unit of frequency x time.\
One PRB is composed on multiple subcarriers, but this level of precision is not pertinent for the representation as it serves mainly for increasing the data output capacity.

Here, the Y axis is composed of 51 PRBs of 180KHz each. This can vary depending on the bandwidth configuration.

#### 8.1.2. Time Axis
The X axis is the time axis. 

- A **frame** is 10ms
- A **Subframe** is 1ms, 1/10th of a Frame.
- The number of **Slot** per Subframe can vary from 2 to 8 depending of the antenna configuration. It is commonly 2 slots per subframe, so 1 slot = 0.5ms
- There is 14 **Symbol** per Slot in 5G. This is the smallest allocation duration.

#### 8.1.3. PHY layer
The Traces from Amarisoft provide enough information through the PHY layer to build most part of the resource grid. (Unfortunately his does apply for PDSCH, where positions are more difficult to find)

Example of a PHY event : 

```10:32:35.169 [PHY] UL 0001 01 4601  476.19 PUSCH: harq=0 prb=15:34 symb=0:14 ...```

From there we can extract information to help us fill the grid : 

| Unit | PHY | Rendering |
|------|-----|-----------|
| PRB | `prb=15:34` | PRBs 15 → 48 |
| Symbol | `symb=0:14` | Symbols 0 → 13 |
| SFN | `476.19` | Frame 476 - Slot 19 |

The **System Frame Number (SFN)** takes only into account the Frame and Slot number, but we can construct the SubFrame number through the number of **Symbol per Frame (SpF)**. \
This gives us : Frame 476, SubFrame 9, Slot 1, Symbol 0-13

**Note:** The System Frame Number (SFN) is a 10-bit value that cycles every 1024 frames and does not include the **Hyper Frame Number (HFN)**. Therefore, we must infer the HFN by detecting SFN wraparound:

```
HFN: N, Frame 1023 → HFN: N+1, Frame 0
```

#### 8.1.4. SSB

There is another type of event which can be easily displayed but which are not in the traces but in the antenna configuration: **SSB**

```SSB: id=0 arfcn=630336 mu=1 L=8 period=20 offset=0 k_ssb=20 prb=15:21 symb=2```

From there we can extract information to help us fill the grid : 

| Unit | SSB | Rendering |
|------|-----|-----------|
| ID | `id=0` | -- |
| PRB | `prb=15:21` | PRBs 15 → 35 |
| Symbol | `symb=2` | Symbol 2 → 2 |
| SFN | `period=20` | Every 20ms (2 frames) |

### 8.2. Architecture Principles

#### 8.2.1. Data Pipeline

As for every panel the events are parsed from the data source (File / Websocket) to the `Trace` format. \
This is where we extract the information from the events and store it in the `PHYInfos` struct based on the rules above. HFN is not present because is cannot be extracted from the PHY event itself.

``` rust
pub struct PHYInfos {
    pub direction: Direction, // Direction (UL or DL)
    pub channel_type: PHYChannelType, /// Channel type (PDSCH, PUSCH, etc.)
    pub frame: u16, /// Frame number
    pub slot: u8, /// Slot number within frame
    pub prb_start: u16, /// PRB start position
    pub prb_length: u16, /// PRB length (number of PRBs)
    pub symb_start: u8, /// Symbol start position within slot
    pub symb_length: u8, /// Symbol length (number of symbols)
}
```
When events are added to the trace, we process the `PHYInfos` and store them in a the `CachedPHYEvent` with the computed HFN.


#### 8.2.2. Event System Integration

The panel implements `EventSubscriber` for reactive updates:

```rust
impl EventSubscriber for ResourceBlocks {
    fn on_event_added(&mut self, event: &Trace, index: usize, _ctx: &EventContext) {
        // Cache PHY events with HFN
        if let AdditionalInfos::PHYInfos(phy) = &event.additional_infos {
            self.add_phy_event(index, phy.clone());
        }
    }
    
    fn on_event_focused(&mut self, event: &Trace, index: usize, _ctx: &EventContext) {
        // Track for highlight and auto-center grid
        self.focused_trace_index = Some(index);
        self.center_on_event(event);
    }
    
    fn on_events_cleared(&mut self) {
        self.clear_slots();
        self.start_limit = (0, 0, 0);
        self.end_limit = (0, 0, 0);
    }
}
```

**Auto-Centering:** When a PHY event is focused in the navigation panel, the Resource Blocks grid automatically scrolls to center on that frame/slot.


#### 8.2.3. Three-Dimensional Grid Model

For performance issues, it is not reasonable to think of building the whole grid : \
1s = 100 Frames = 1k SubFrames = 2k Slots = 28k Symbols \
One symbol column is one state ID of 8 bits x 51 PRBs = 51 octets \
So 1s = 1.39 Mo, 1 hour of trace is over 5Go, and this is without counting for indexing cost and other additional data per symbol.

Therefore, we need to build the grid on demand, only for the visible part of the trace.
We render N=10 frames before and after the focused event. With the possibility to navigate 1 Frame or 1 SubFrame. This also allows us to reduce the hell of scrolling headlessly on an infinite time axis.



The grid operates on three axes with distinct addressing strategies:

| Axis | Dimension | Addressing | Rendering |
|------|-----------|------------|-----------|
| **Y** | Frequency (PRBs) | 0 to n_rb_dl (e.g., 0-50) | Vertical scroll with sticky labels |
| **X** | Time (Symbols/Slots) | Symbol: 0-13 per slot<br>Slot: aggregated | Horizontal scroll with sticky headers |
| **X"** | Frame Window | Global slot index | Virtual scrolling with 200-slot (10 frames) window |

**Global Slot Addressing:**

We use the absolute slot position for indexing: 

```rust
// HFN-aware slot addressing for 1024-frame looping
pub type SlotAddress = (u32, u16, u16);  // (hfn, frame, slot)

// Global slot index for window positioning
let global_slot = (hfn as usize * 1024 + frame as usize) * slots_per_frame + slot;
```

#### 8.2.3 HFN-Aware Event Caching

5G NR frames loop every 1024 frames. Without Hyper Frame Number tracking, frame 0 at startup is indistinguishable from frame 0 after wraparound.

```rust
pub struct CachedPHYEvent {
    pub trace_index: usize,
    pub hfn: u32,  // Hyper Frame Number increments on wraparound
    pub phy_info: PHYInfos,
}

// Limits stored with HFN for correct range checking
pub struct ResourceBlocks {
    pub start_limit: (u32, u16, u16),  // (hfn, frame, slot)
    pub end_limit: (u32, u16, u16),
}
```

**Frame Wrap Detection:**
```rust
// Detect wraparound by large frame jump (more than -512 frames)
if frame < end_frame && (end_frame - frame) > 512 { // Wrapped: 1023 -> 0
    hfn = hfn.saturating_add(1);  // HFN += 1
}
```

However, because of the precision of the Amarisoft logger, we can have prior events (relative to the SFN) coming after the current event. So we need to be able to detect an anti-wrap.

```rust
// Detect anti-wraparound by large frame jump (more than +512 frames)
if frame > end_frame && (frame - end_frame) > 512 { // Anti-wrapped: 0 -> 1023
    hfn = hfn.saturating_sub(1); // HFN -= 1
}
```

### 8.3. Data Structures

#### 8.3.1 SlotGrid
```rust
pub struct SlotGrid {
    pub hfn: u32,               // HyperFrame number
    pub frame_number: u16,      // Frame (0-1023, wraps around)
    pub subframe_number: u16,   // Subframe (0-9)
    pub slot_number: u16,       // Slot within frame
    pub symbols: Vec<Vec<ResourceType>>,        // [prb][symbol] -> Resource Type 
    pub event_indices: Vec<Vec<Option<usize>>>, // [prb][symbol] -> trace index
    pub ssb_ids: Vec<Vec<Option<usize>>>,       // [prb][symbol] -> SSB index
}
```

#### 8.3.2 Resource Types
| Type | Color | Purpose |
|------|-------|---------|
| Empty | White | Unallocated |
| PDCCH | Dark green `0, 100, 0` | Downlink control |
| PUCCH | Light green `0, 200, 0` | Uplink control |
| PDSCH | Blue `0, 100, 200` | Downlink data |
| PUSCH | Cyan `0, 180, 180` | Uplink data |
| PRACH | Yellow `255, 220, 0` | Random access |
| SSB | Magenta `200, 0, 200` | Synchronization signal |
| DMRS | Red `200, 0, 0` | Reference signal |
| Guard | Dark gray `100, 100, 100` | Guard band |

### 8.4. UI Rendering

#### 8.4.1 Dual View Mode Architecture

The panel supports two visualization modes with shared underlying data:

```rust
pub enum ViewMode {
    Symbol,  // Detailed: 14 symbols × PRBs per slot
    Slot,    // Aggregated: 1 column per slot with priority-based color
}
```

**View Mode Characteristics:**

| Aspect | Symbol View | Slot View |
|--------|-------------|-----------|
| **Use Case** | Detailed analysis | Overview, pattern detection |
| **Cell Count** | 14 × PRBs × slots | PRBs × slots |
| **Resource Display** | Exact symbol allocation | Highest priority resource |
| **Performance** | Higher CPU (more cells) | Optimized (cached aggregation) |

**Priority Aggregation for Slot View:**

In case many resources are allocated to the same PRB and slot, we need to prioritize the most important one to be displayed.
```rust
// cf
impl ResourceType -> pub fn priority(&self)
```

#### 8.4.2 Sticky Header Rendering

Traditional `egui::Grid` cannot handle both sticky row headers (PRB labels) and sticky column headers (slot info). The panel uses a custom overlay approach:

**Implementation:**
- Single `ScrollArea::both()` contains the entire grid
- Headers/labels drawn as overlays using `painter` with viewport-relative coordinates
- Content rect calculations account for label/header dimensions

#### 8.4.3. Visible-Range Culling

Only render cells within the visible viewport for performance:

```rust
// Calculate visible ranges from scroll offset
let vis_start_slot = (scroll_offset_x / slot_width) as usize;
let vis_end_slot = ((scroll_offset_x + viewport_width) / slot_width) as usize + 1;
let vis_start_prb = (scroll_offset_y / cell_size) as usize;
let vis_end_prb = ((scroll_offset_y + viewport_height) / cell_size) as usize + 1;

// Clamp to actual data bounds
let vis_start_slot = vis_start_slot.min(total_slots);
let vis_end_slot = vis_end_slot.min(total_slots);
```

#### 8.4.4. Smart Border Drawing

Vertical-only borders with thickness indicating hierarchy:

| Border Type | Thickness | Purpose |
|-------------|-----------|---------|
| Symbol gap | 0.5px | Visual separation within slot |
| Slot boundary | 1.0px | Major time unit separation |
| Subframe gap | 2.0px | Frame structure boundary |

**No horizontal borders** - visual clarity through vertical separation only.

#### 8.4.5. Auto-Scroll & Focus Highlighting

Recentering the data window (`start_slot`) on a focused event is not enough: `egui::ScrollArea` keeps its own independent scroll offset, so the focused slot could still be rendered far outside the visible viewport, and switching `ViewMode` (Symbol ↔ Slot) changes `slot_width` drastically, invalidating any previous scroll position.

**Fix - explicit scroll request:**

```rust
// Set whenever focus changes or the view mode is toggled
self.scroll_pending = true;

// Inside the ScrollArea closure, once per pending request:
if self.scroll_pending {
    if let Some(slot_idx) = focused_global_slot.and_then(|g| g.checked_sub(self.start_slot)) {
        let target_rect = Rect::from_min_size(Pos2::new(slot_x, y_top), Vec2::new(slot_width, y_height));
        ui.scroll_to_rect(target_rect, Some(egui::Align::Center));
        self.scroll_pending = false;
    }
}
```

`Ui::scroll_to_rect` must be called from within the `ScrollArea`'s child `Ui` - it queues an animated scroll to bring the given rect into view regardless of the current offset, which fixes the "wrong X position until manual scroll" issue.

**Focus legend:** the header (slot/frame label) and PRB label corresponding to the focused event are highlighted with a translucent `theme.accent` fill + border, so the current position remains identifiable even after scrolling away:

| Element | Highlight condition |
|---------|----------------------|
| Slot/Frame header | `global_slot == focused_global_slot` |
| PRB label | `prb` within `[prb_start, prb_start + prb_length - 1]` |

### 8.5. Performance Optimizations

#### 8.5.1. Hot Path Optimizations

| Optimization | Before | After | Impact |
|--------------|--------|-------|--------|
| Limit checks | Per cell (14×PRBs×slots) | Per slot | -93% ops |
| Slot lookups | 3× per PRB | 1× cached | -85% lookups |
| Symbol passes | 3 separate loops | 1 combined | -67% iterations |
| Coordinate math | Per-cell computation | Hoisted to loop start | -99% redundant math |
| Tooltip strings | Allocated every frame | Deferred to callback | -95% idle allocations |

**Key Techniques:**

1. **Hoisted Coordinate Calculations:**
```rust
// Before: computed per cell (N × M × 14 times)
let y = content_rect.top() + header_height + (prb as f32 * cell_size);

// After: computed once
let grid_origin_y = content_rect.top() + header_height;
// In loop: just add offset
let y = grid_origin_y + (prb as f32 * cell_size);
```

2. **Early Exit Aggregation:**
```rust
// Inlined with early termination
for symb in 0..symbols_per_slot {
    let resource = slot.get(prb, symb);
    if resource.priority() >= 6 {  // PUSCH/PDSCH
        return *resource;  // Early exit, no need to scan remaining
    }
}
```

3. **Deferred Tooltip Allocation:**
```rust
// Before: always allocates
response.on_hover_text(format!("Frame {}", frame));

// After: allocates only when shown
response.on_hover_ui(|ui| {
    ui.label(format!("Frame {}", frame)); // Only runs on hover
});
```

### 8.6. Files

- `tramex/src/panels/resources_blocks.rs` - Panel implementation
- `tramex-tools/src/interface/layer.rs` - PHY layer visibility (default: on)

### 8.7. Usage

1. Load a file with PHY traces (PDSCH, PUSCH, PUCCH, PRACH)
2. Open Resource Blocks panel from Windows menu
3. Navigate events - grid auto-centers on focused PHY event
4. Toggle Symbol/Slot view for detail vs overview
5. Hover cells for detailed PHY allocation info
6. Adjust cell size for visibility of dense allocations

---

## 9. HARQ Panel - PHY Layer Visualization

#### Implementation Date
2026-03-12
2026-09-01 - Added filtering by HARQ process & Click to event

### 9.0. Overview

The HARQ panel provides a chronograph-style visualization of PHY layer events, showing the communication flow between UE (User Equipment) and BST (Base Station). It displays PDCCH grants, PDSCH/PUSCH data transfers, and PUCCH feedback with color-coded HARQ process identification.


#### 9.0.1. Definitions
- **ndi**: New Data Indicator, 0 = new data, 1 = redundant transmission
- **retx**: Retransmission count, 0 = first transmission, >0 = retransmission
- **rv_idx**: Redundancy Version index, 0-3 for different redundancy versions
- **crc**: Cyclic Redundancy Check, "OK" or "FAIL" for error detection

#### 9.0.2. Purpose
- Visualize the HARQ (Hybrid Automatic Repeat Request) process flow
- Track retransmissions via `retx`, `rv_idx`, and `crc` fields
- Understand ACK/NACK feedback timing
- Identify scheduling patterns by HARQ process number

#### 9.0.3. Supported Channels

| Channel | Direction | Role in HARQ |
|---------|-----------|--------------|
| **PDCCH** | DL | Control channel carrying DCI grants |
| **PDSCH** | DL | Downlink data (carries TB from gNB to UE) |
| **PUSCH** | UL | Uplink data (carries TB from UE to gNB) |
| **PUCCH** | UL | Uplink control (ACK/NACK feedback) |

### 9.1. Panel Layout

```
┌─────────────────────────────────────────────────────────┐
│                    UE              BST                  │
│                    │                │                   │
│  PDCCH dci=1_1     │ ◄──────────────┤  (DL grant)       │
│  PDSCH harq=0      │ ◄──────────────┤  (DL data)        │
│  PUCCH format=1    ├───────────────►│  ACK              │
│  PDCCH dci=0_1     │ ◄──────────────┤  (UL grant)       │
│  PUSCH harq=0      ├───────────────►│  crc=OK           │
│                    │                │                   │
├─────────────────────────────────────────────────────────┤
│  HARQ: [0] [1] [2] ...              (color legend)      │
└─────────────────────────────────────────────────────────┘
```

- **Vertical lifelines**: UE on left, BST on right
- **Arrows**: Direction indicates UL (→) or DL (←)
- **Labels**: Channel-specific info above each arrow
- **Legend**: Shows active HARQ processes with their colors

### 9.2. Arrow Display

#### 9.2.1. Arrow Labels by Channel

| Channel | Label Format | Example |
|---------|--------------|---------|
| **PDCCH** | `PDCCH dci={format} ndi={} rv_idx={}` | `PDCCH dci=1_1 ndi=1 rv_idx=0` |
| **PDSCH** | `PDSCH harq={} retx={} rv_idx={}` | `PDSCH harq=0 retx=0 rv_idx=0` |
| **PUSCH** | `PUSCH harq={} retx={} rv_idx={} crc={}` | `PUSCH harq=0 retx=0 rv_idx=0 crc=OK` |
| **PUCCH** | `PUCCH format={} {ACK/NACK}` | `PUCCH format=1 ACK` |

#### 9.2.2. Color Coding

- **HARQ Process Colors** : Each Harq gets its own color to help the diferenciation
- **PUCCH**: Uses neutral theme color (no HARQ process ID)
- **Focused arrow**: Highlighted with semi-transparent background bar
- Support Theme Colors

#### 9.2.3. Focus & Navigation

- When an event is focused in the navigation panel the corresponding arrow is highlighted with a colored background
- It is possible to click on an arrow to focus on the corresponding event in the navigation panel

### 9.3. Filtering Logic

The panel automatically filters out non-HARQ events:

| Filtered Out | Reason |
|--------------|--------|
| `harq=si` | MIB/SIB broadcasts (system info, no HARQ) |
| PUCCH format!=1 | CSI (Channel State Info) only, no HARQ feedback |
| PDCCH without `harq_process` | DCI 1_0 for SIB (no HARQ) |
| PRACH | Random access, not HARQ |

If one harq process has been selected, only the corresponding arrows will appear in the panel.

### 9.4. PHY Trace Parsing

#### 9.4.1. PHYChannelData Structure

Channel-specific fields are stored in a dedicated enum:

```rust
pub enum PHYChannelData {
    Pdcch { 
        dci: String,                      // "0_1" or "1_1"
        harq_process: Option<u8>, 
        ndi: Option<u8>,                  // New Data Indicator
        rv_idx: Option<u8>,               // Redundancy Version
        harq_feedback_timing: Option<u8>  // DCI 1_1 only
    },
    Pdsch { retx: Option<u8>, rv_idx: Option<u8> },
    Pusch { retx: Option<u8>, rv_idx: Option<u8>, crc: Option<bool>, ack: Option<bool> },
    Pucch { format: Option<u8>, ack: Option<bool> },
    None,
}
```

#### 9.4.2. Multi-Line PDCCH Parsing

PDCCH traces from Amarisoft span multiple lines:

```
10:32:35.169 [PHY] DL 0001 01 476.19 PDCCH: ss_id=2 cce_index=6 al=2 dci=1_1 k1=4
    harq_process=0
    ndi1=1
    rv_idx1=0
    harq_feedback_timing=1
```

The parser handles continuation lines with `parse_phy_lines()`.

#### 9.4.3. DCI Format Differences

| DCI Format | Direction | Specific Fields |
|------------|-----------|-----------------|
| **0_1** | UL grant | NA |
| **0_1** | UL grant with RNTI | `ndi=`, `rv_idx=` |
| **1_0** | DL grant with RNTI | NA |
| **1_1** | DL grant | `ndi1=`, `rv_idx1=`, `harq_feedback_timing=` |

#### 9.4.4. Format & Inferences from Amarisoft

Because the traces are from the BST point of vue, the informations availabled in PUSCH or PUCCH 
traces are not all sent fromt the UE. 



### 9.5. Performance

#### 9.5.1. Arrow Buffer

The panel keeps a sliding buffer of at most `MAX_ARROWS` arrows for memory efficiency.
When the limit is reached, the oldest half is dropped:

```rust
const MAX_ARROWS: usize = 100;

if self.arrows.len() >= MAX_ARROWS {
    self.arrows.drain(0..50);  // Remove oldest 50
}
```

#### 9.5.2. Buffer Regeneration on Navigation

Because arrows outside the buffer are discarded, navigating back to an older event
would otherwise show nothing. `on_event_focused` therefore rebuilds the window on demand:

- `buffered_range()` returns the `(min, max)` trace index currently held in `arrows`.
- If the focused index is **before** that range, the arrows were dropped → rebuild.
- If it is **after** the range, a rebuild happens only when a PHY arrow actually exists
  in between (avoids useless rebuilds while stepping through non-PHY traces).
- `rebuild_window(all_events, index)` walks backwards from the focused event until
  `MAX_ARROWS / 2` arrows are collected, then forwards up to `MAX_ARROWS`, recomputes
  the HFNs sequentially (anti-wrap state is reset first) and re-sorts by `(hfn, frame, slot)`.

#### 9.5.3. Event System Integration

The panel implements `EventSubscriber`:
- `on_event_added`: Creates arrow if PHY event matches criteria
- `on_event_focused`: Updates highlight, triggers auto-scroll, and rebuilds the arrow
  buffer if the focused event lies outside it
- `on_events_cleared`: Clears all arrows and resets the HFN tracking state


### 9.7. Files

- `tramex/src/panels/harq_panel.rs` — Panel implementation
- `tramex-tools/src/interface/parser/parser_phy.rs` — PHY parsing with `PHYChannelData`

### 9.8. Usage

1. Load a file containing PHY layer traces
2. Open HARQ panel from Windows menu
3. Navigate through events — panel auto-scrolls to focused PHY event
4. Identify HARQ processes by color
5. Track retransmissions by watching `retx`, `rv_idx` changes
6. Monitor ACK/NACK feedback in PUCCH/PUSCH arrows

---

## 10. AI Explain Feature

#### Implementation Date
2026-03-20

### 10.0. Overview
#### 10.0.1. Problem Solved
- Understanding complex 4G/5G protocol traces requires deep domain knowledge
- Users need quick explanations of what each trace represents without consulting external documentation
- No built-in assistance for interpreting RRC, NAS, NGAP, and other protocol layers

#### 10.0.2. Solution: AI-Powered Trace Explanation
Integrated an AI chatbot feature that sends trace context to Mistral AI (with extensible connector architecture) and displays a concise explanation directly in the Messages panel.

#### 10.0.3. Benefits
- **Instant context**: Get human-readable explanations of any trace with one click
- **Educational**: Learn protocol behavior as you analyze traces
- **Extensible**: Connector trait supports multiple AI providers
- **Secure**: API key loaded from environment or settings, never committed to git

### 10.1. Architecture

#### 10.1.1. AI Connector System (`tramex-tools` crate)

**Trait-based design** for multiple providers:

```rust
pub trait AIConnector: Send {
    fn build_request(&self, trace: &Trace, api_key: &str) -> Result<AIRequest, TramexError>;
    fn parse_response(&self, response_body: &str) -> Result<String, TramexError>;
}
```

**AIProvider enum** with factory function:
```rust
pub enum AIProvider {
    Mistral,
    // Future: OpenAI, Anthropic, etc.
}

pub fn create_connector(provider: &AIProvider) -> Box<dyn AIConnector> {
    match provider {
        AIProvider::Mistral => Box::new(MistralConnector::default()),
    }
}
```

#### 10.1.2. Request/Response Flow

```
User clicks "AI Explain"
    ↓
MessageBox.request_ai_explain()
    ↓
MistralConnector.build_request(trace, api_key)
    ↓
ehttp::fetch() async request
    ↓
poll_promise::Promise polls for response
    ↓
MistralConnector.parse_response()
    ↓
AIExplainStatus::Done(explanation)
    ↓
UI displays explanation in scrollable text area
```

#### 10.1.3. Prompt Engineering

**System prompt** (Mistral `mistral-medium-latest`):
- Concise telecom expert persona
- Structured context format with layer, timestamp, direction, channel, and raw text
- Bullet-point output format
- Key fields identification
- Brief 5-7 sentence summary

**Request structure**:
```rust
AIRequest {
    url: "https://api.mistral.ai/v1/chat/completions",
    headers: {"Authorization": "Bearer {api_key}"},
    body: JSON with system prompt + user context,
}
```

### 10.2. UI Integration

#### 10.2.1. Settings Menu

New **Settings** menu added to top bar (next to Menu, Windows, About):

```
┌─────────────────────────────────────────┐
│ Menu │ Windows │ Settings │ About      │
└─────────────────────────────────────────┘
              ↓
        ┌──────────────────┐
        │ AI Configuration │
        └──────────────────┘
              ↓
        ┌─────────────────────────────┐
        │  AI Configuration             │
        │  ─────────────────────────    │
        │  Provider: [Mistral ▼]        │
        │                               │
        │  API Key: [sk-xxxxxxxxxxxx]   │
        │                               │
        │  [Clear]         [Save]       │
        │                               │
        │  Loaded from env: Yes/No      │
        └─────────────────────────────┘
```

#### 10.2.2. Messages Panel Integration

**AI explain button** appears below "Show full message":

```
┌─────────────────────────────────────────┐
│ Layer: RRC                              │
│ AdditionalInfos(...)                    │
│ [ ] Show full message                   │
│ ─────────────────────────────────────   │
│ 🤖 AI Explain           ✓              │  ← AI section
└─────────────────────────────────────────┘
              ↓ (after clicking)
┌─────────────────────────────────────────┐
│ 🤖 AI Explain          ✓               │
│ ─────────────────────────────────────   │
│ • Message Type: RRCConnectionRequest      │
│ • Direction: Uplink (UE → eNB)          │
│ • Purpose: UE initiates connection      │
│ • Key Fields: initial UE identity,      │
│   establishment cause                   │
│ • Summary: This is the first message    │
│   the UE sends when trying to connect   │
└─────────────────────────────────────────┘
```

#### 10.2.3. State Management

**AIExplainStatus enum**:
```rust
pub enum AIExplainStatus {
    Idle,           // No request made yet
    Loading,        // Request in flight (spinner shown)
    Done(String),   // Successful explanation
    Error(String),  // HTTP or parsing error
}
```

**State transitions**:
- `Idle → Loading`: User clicks button
- `Loading → Done`: Response received successfully
- `Loading → Error`: HTTP error or invalid response
- `Done/Error → Idle`: User navigates to different trace

### 10.3. Configuration

#### 10.3.1. Environment Variable

Set `TRAMEX_AI_API_KEY` before running:
```bash
export TRAMEX_AI_API_KEY="sk-xxxxxxxxxxxxxxxx"
cargo run
```

Or in `.env` file (gitignored):
```
TRAMEX_AI_API_KEY=sk-xxxxxxxxxxxxxxxx
```

#### 10.3.2. Runtime Configuration

Settings are passed from `TramexApp` → `FrontEnd` → `Application` → `EventBus` → all panels:

```rust
// In TramexApp::update()
self.frontend.set_ai_config(&self.ai_settings.api_key, &self.ai_settings.provider);

// EventSubscriber trait method
fn set_ai_config(&mut self, key: &str, provider: &AIProvider) {
    self.ai_api_key = key.to_string();
    self.ai_provider = provider.clone();
}
```

### 10.4. Feature Flag

The AI feature is **enabled by default** via `default = ["websocket", "ai"]` in `tramex/Cargo.toml`.

To build without AI:
```bash
cargo build --no-default-features --features websocket
```

### 10.5. Files Modified

**New Files:**
- `tramex-tools/src/ai/mod.rs` — `AIConnector` trait, types, factory
- `tramex-tools/src/ai/mistral.rs` — `MistralConnector` implementation
- `tramex/src/ai_settings.rs` — `AISettings` with UI and env loading

**Modified Files:**
- `tramex-tools/Cargo.toml` — Added `ai` feature
- `tramex-tools/src/lib.rs` — Registered `ai` module
- `tramex/Cargo.toml` — Added `ai` to default features, `poll-promise` dependency
- `tramex/src/lib.rs` — Registered `ai_settings` module
- `tramex/src/app.rs` — Settings menu, AI config window, config forwarding
- `tramex/src/frontend.rs` — `set_ai_config()` forwarding
- `tramex/src/event_system/event_bus.rs` — `set_ai_config` trait method + forwarding
- `tramex/src/event_system/application.rs` — `set_ai_config()` forwarding
- `tramex/src/panels/panel_message.rs` — AI button, async request, response display
- `.gitignore` — Added `.env`

### 10.6. Future Enhancements

1. **Multiple providers**: OpenAI GPT-4, Anthropic Claude, local LLMs via Ollama
2. **Context history**: Include previous traces in prompt for richer explanations
3. **Custom prompts**: User-defined system prompts for specific use cases
4. **Caching**: Store explanations to avoid duplicate API calls
5. **Offline mode**: Pre-trained model on-device for disconnected operation
6. **Multi-language**: Translated explanations for international users
