# Development Log

## Overview
This document tracks major improvements to Tramex, including the indexed file navigation system, automatic batch continuation, and enhanced UI with a dedicated navigation panel.

---

## 1. Indexed Pre-Scan with Lazy Parsing

### Implementation Date
2025-10-10

### Problem Solved
- Slow navigation through large log files (900k+ events)
- Landing on filtered/disabled layers when navigating
- No visibility into total event count or progress

### Solution: File Index System
Implemented a lightweight index that pre-scans the file to identify all log boundaries and extract metadata without full parsing.

#### Key Components

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

### Performance Improvements
- **Index building**: ~3-4x faster (8s → 2-3s for 900k traces)
- **Navigation**: Instant jumps to any event position
- **Memory**: Only parsed events are cached
- **Filtering**: Can skip disabled layers efficiently

### Files Modified
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

### Implementation Date
2025-10-10

### Problem Solved
When navigating with layer filters enabled, the "Next" button would sometimes stop after loading 5-6 batches without finding an enabled layer, requiring multiple clicks to continue.

### Solution: Loop-Based Batch Loading
Modified the frontend to keep loading batches in a loop until an enabled layer is found or the end of file is reached.

#### How It Works
1. User clicks "Next"
2. Search through currently loaded events
3. If no enabled layer found → load next batch
4. Search through new batch
5. If still not found → load another batch
6. **Repeat until**: enabled layer found OR end of file OR error

#### Implementation Details
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

### Benefits
- **No more manual clicking**: One click finds the next enabled event
- **Faster navigation**: Automatic batch loading feels instant
- **Better UX**: Users don't need to understand batching

### Files Modified
- `tramex/src/frontend.rs` - Added while loop for batch continuation
- `tramex/src/panels/trame_manager.rs` - Navigation logic with flags

---

## 3. Enhanced Navigation UI

### Implementation Date
2025-10-10

### Problem Solved
- Limited visibility into navigation state
- No clear indication of progress or total events
- Plain, unstyled controls

### Solution: Dedicated Navigation Panel + Enhanced Controls

#### New Navigation Panel Window (`navigation_panel.rs`)
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

**Color Coding:**
- Blue (`50, 120, 220`) - Next button
- Brown (`150, 100, 50`) - Previous button
- Orange (`255, 200, 100`) - Loaded count
- Purple (`200, 150, 255`) - Timestamps
- Red (`255, 150, 150`) - Layer info

**Features:**
- Opens by default with other windows
- Can be toggled via Windows menu
- Resizable and movable
- Real-time updates as you navigate

#### Button Styling Improvements
- **Start button**: Green (`50, 180, 50`) - indicates "begin"
- **Next button**: Blue (`50, 120, 220`) - indicates "continue"
- **Previous button**: Brown (`150, 100, 50`) - indicates "go back"
- **Rounded corners**: 8px border radius for modern look
- **Centered layout**: Better visual balance

#### Menu Organization
Moved Windows menu to top menu bar:
- **Menu**: Organize windows, About, Quit
- **Windows**: Toggle all panels (Navigation, Message Box, etc.)
- **About**: Documentation links

### Files Modified
**New Files:**
- `tramex/src/panels/navigation_panel.rs`

**Modified Files:**
- `tramex/src/panels/mod.rs` - Added navigation_panel module
- `tramex/src/frontend.rs` - Added NavigationPanel, updated menu integration
- `tramex/src/app.rs` - Integrated Windows menu in top bar
- `tramex/src/panels/panel_message.rs` - Re-added "Show full message" checkbox

---

## 4. UI/UX Improvements Summary

### Visual Design Principles
- **Color coding**: Consistent colors for same information types
- **High contrast**: White text on colored buttons
- **Clear icons**: Universal symbols (▶, ◀)
- **Grouped sections**: Related info together
- **Responsive**: Adapts to window size

### Accessibility Features
1. **High contrast**: White text on colored buttons
2. **Clear icons**: Universal symbols
3. **Consistent colors**: Same meaning throughout
4. **Large buttons**: 150x40px minimum for easy clicking
5. **Monospace timestamps**: Easy to read and compare

### User Experience Benefits
- **Better visibility**: Color-coded information is easier to scan
- **Clearer status**: Icons and colors show state at a glance
- **Organized layout**: Grouped sections reduce visual clutter
- **Instant feedback**: Real-time updates as you navigate
- **No more stuck navigation**: Automatic batch loading

---

## Testing Checklist

### File Index System
- [x] Open a file - verify index builds quickly
- [x] Check total event count displays correctly
- [x] Navigate through logs with layer filters
- [x] Verify automatic batch loading works
- [x] Test with large files (900k+ events)

### Navigation UI
- [x] Navigation panel opens by default
- [x] Statistics display correctly
- [x] Buttons are centered and styled
- [x] Colors display correctly
- [x] Start button is green, Next is blue
- [x] Timestamp and layer update in real-time
- [x] Windows menu appears in top bar

### Batch Continuation
- [x] Click Next with filters - automatically finds enabled layer
- [x] No manual clicking needed for multiple batches
- [x] Stops at end of file correctly

### WebSocket Mode
- [ ] WebSocket connection still works
- [ ] Navigation works without total count
- [ ] No errors from missing index

---

## Future Enhancements

### Navigation Features
1. **Jump to Event**: Input field to jump to specific event number
2. **Keyboard shortcuts**: Arrow keys for navigation
3. **Event markers**: Visual indicators for important events
4. **Timeline view**: Visual timeline of events
5. **Search**: Search through events by content
6. **Bookmarks**: Mark and save important event positions

### Performance
1. **Parallel indexing**: Build index in background thread
2. **Incremental updates**: Update index as file grows
3. **Smart caching**: LRU cache for parsed events
4. **Batch size optimization**: Adaptive batch sizes

### UI/UX
1. **Filtering UI**: Visual filter controls in navigation panel
2. **Statistics graphs**: Charts showing event distribution
3. **Performance metrics**: Show parsing speed, memory usage
4. **Batch controls**: Skip forward/backward by N events
5. **Auto-play**: Automatically advance through events
6. **Dark/Light themes**: Theme support

---

## Notes

- All changes are backward compatible
- No breaking changes to existing functionality
- WebSocket mode continues to work unchanged
- Colors and icons can be easily customized
- Navigation panel can be hidden if not needed
- File index is built once and cached
---

## UI Layout Reference

### Navigation Panel Window Layout
```
┌─────────────────────────────────────────┐
│  Navigation                     [X]     │
├─────────────────────────────────────────┤
│  Event: 1 / 1234  |  Loaded: 150       │
│  Time: 12:34:56.789  |  Layer: RRC     │
│  ─────────────────────────────────────  │
│         [  ▶ Next  ]  [  ◀ Previous  ] │
└─────────────────────────────────────────┘
```

### Color Reference
| Element | Color (RGB) | Usage |
|---------|-------------|-------|
| Next button | `50, 120, 220` | Blue - Continue action |
| Start button | `50, 180, 50` | Green - Begin action |
| Previous button | `150, 100, 50` | Brown - Go back |
| Event count | `100, 200, 255` | Blue - Current position |
| Loaded count | `255, 200, 100` | Orange - Memory usage |
| Timestamp | `200, 150, 255` | Purple - Time info |
| Layer | `255, 150, 150` | Red - Layer type |

### Menu Structure
```
Top Menu Bar:
  [Theme] | [Menu ▼] [Windows ▼] [About ▼]
  
Windows Menu:
  ☑ Navigation
  ☑ Message Box
  ☑ Logical Channels
  ☑ Link Panel
```

---

## 5. RRC Field Viewer & Message Panel Improvements

### Implementation Date
2025-10-10

### Problem Solved
- No dedicated panel for viewing specific RRC message fields
- Message panel displayed too much information (timestamp, hex always visible)
- Array values in JSON were not properly displayed
- Technology information was not visible in logical channels panel
- Technology detection relied only on file headers (which could be missing)

### Solution: RRC Field Viewer + UI Enhancements

#### New RRC Field Viewer Panel (`bst_config.rs`)
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

#### Message Panel Improvements
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

#### Technology Detection Enhancement
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

#### Logical Channels Panel Update
**New display:**
```
[----] [----] [----] [Downlink] [----] [----] [----]
[    ] [Techno: LTE (4G)] [    ] [----] [Uplink] [----]
```

Shows technology (LTE/NR) in the channel grid for quick reference.

### Files Modified

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

### Technical Details

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

### Benefits

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

### Future Enhancements

1. **Dynamic field configuration**: Load field mappings from config file
2. **More message types**: Add configurations for other RRC messages (SIB2, MIB, etc.)
3. **Field filtering**: Show/hide specific fields
4. **Export fields**: Copy field values to clipboard
5. **Field history**: Track field value changes over time
6. **Comparison view**: Compare fields across multiple messages

---

# WebSocket Connection Debugging Guide

## Current Situation

You're receiving the initial "ready" message from the Amarisoft server but not seeing subsequent log messages.

## Root Cause

**The Amarisoft WebSocket server uses a REQUEST-RESPONSE pattern**, not a push model:

1. ✅ Server sends "ready" message when you connect
2. ❌ **You must send `log_get` requests to receive logs** (this is not happening automatically)
3. ❌ Server only sends log data in response to your `log_get` requests

## How Your Application Works

### Message Flow:
```
1. Connect → Server sends "ready" message
2. User navigates (clicks Next) → Triggers `should_get_more_log = true`
3. `get_more_data()` is called → Sends `log_get` request to server
4. Server responds → `try_recv()` receives the log data
```

### The Problem:
- `try_recv()` is called every frame to check for incoming messages ✅
- `get_more_data()` sends the request to the server ✅
- **BUT** `get_more_data()` is only called when:
  - You click the "Next" navigation button
  - You're near the end of loaded events (preloading)
  - `should_get_more_log` is set to `true`

## Debugging Steps

### Step 1: Check the Logs

I've added comprehensive logging to help you debug. Run your application and look for these log messages:

```
✅ WebSocket connection opened successfully
✅ Received 'ready' message from server: ENB
💡 Server is ready. You need to click 'Load More' or enable auto-loading to request logs.
```

### Step 2: Trigger a Request

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

### Step 3: Watch for Server Response

After sending a request, you should see:
```
🔵 WebSocket event received: Message(...)
📨 Raw WebSocket text message: {"message":"log_get","logs":[...]}
```

If you DON'T see the server response, the problem is with the server or network.

## Common Issues

### Issue 1: "Screens" on Amarisoft Server

Amarisoft servers often require you to enable "screens" to receive log data:

**Check if screens are enabled:**
```bash
# Connect to your Amarisoft VM
ssh user@137.194.194.36

# Check screen configuration
# Look for screen settings in your ENB configuration file
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

### Issue 2: Layer Filters

Your `log_get` request includes layer filters. Make sure:
1. The layers you're requesting are enabled on the server
2. The layers match what the server is configured to send

**Check your layer configuration:**
- Open the Options panel in your app
- Make sure at least one layer is enabled (e.g., RRC, NAS, S1AP)

### Issue 3: Server Not Sending Data

The server might not have any data to send if:
- No UE (User Equipment) is connected
- No traffic is being generated
- The layers you're requesting have no activity

## Testing with a Simple Request

You can test the WebSocket connection manually using a WebSocket client:

```javascript
// Connect to the server
const ws = new WebSocket('ws://137.194.194.36:9001');

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

## Next Steps

1. **Run your application** and check the logs for the emoji indicators (🔵 📨 📤 ✅)
2. **Click the Next button** to trigger a `log_get` request
3. **Check if you see the request being sent** (📤 messages)
4. **Check if you receive a response** (🔵 and 📨 messages)
5. **If no response**, check the Amarisoft server configuration for screen settings

## Files Modified

I've added logging to:
- `tramex-tools/src/interface/websocket/ws_connection.rs`
  - Line 115: Log all WebSocket events
  - Line 122: Log all raw text messages
  - Line 74-77: Log all outgoing requests
  - Line 148-149: Log ready message with helpful hint

All logs use `log::info!()` so they'll be visible by default.

---

## 6. Event-Driven Architecture (Observer Pattern)

### Implementation Date
2025-11-09

### Overview
Refactored the application to use an event-driven architecture with the Observer pattern, replacing the legacy polling-based system with a reactive, notification-based approach.

### Architecture

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

### Data Flow

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

### Key Benefits

| Aspect | Old System | New System |
|--------|-----------|------------|
| **Event Processing** | Loop through all events on navigation | Process once when added |
| **Data Ownership** | Scattered across Data, TrameManager, panels | Single EventStore |
| **Navigation** | Coupled with processing | Separate: data vs. UI focus |
| **Adding Panels** | Manually wire in multiple places | Implement EventSubscriber trait |
| **Testing** | Hard to test individual parts | Easy to mock components |

### API Examples

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

### Panel Implementation

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

### Performance Characteristics

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

### Files Modified

**New Files:**
- `tramex/src/event_system/mod.rs` - Module exports
- `tramex/src/event_system/application.rs` - Application controller
- `tramex/src/event_system/event_bus.rs` - Observer pattern dispatcher
- `tramex/src/event_system/event_store.rs` - Event data management
- `tramex/src/event_system/event_subscriber.rs` - Subscriber trait
- `tramex/src/event_system/data_source.rs` - DataSource trait + FileSource
- `tramex/src/event_system/websocket_source.rs` - WebSocket DataSource
- `tramex/src/event_system/integration.rs` - Helper functions

**Modified Files:**
- `tramex/src/frontend.rs` - Integrated Application controller
- `tramex/src/panels/*.rs` - All panels implement EventSubscriber
- `tramex/src/lib.rs` - Added event_system module

### Migration Notes

The migration preserved backward compatibility by:
1. Keeping legacy `Data` structure for file/WebSocket handlers
2. Transferring events from `Data` to `Application` as bridge
3. Panels implement both old (`PanelController`) and new (`EventSubscriber`) traits
4. Gradual removal of legacy code paths

### Future Enhancements

1. **Complete DataSource migration**: Replace legacy file/WebSocket handlers
2. **Remove Data bridge**: Direct DataSource → Application flow
3. **Panel UI separation**: Remove legacy `show()` method, use only `EventSubscriber`
4. **Advanced features**:
   - Event filtering at Application level
   - Event search and bookmarks
   - Timeline visualization
   - Export/import event sets

---

## 7. Trace Association System

### Implementation Date
2025-01-11

### Overview
A flexible, rule-based system for linking related protocol messages across layers, enabling visibility into parent/child relationships between traces.

### Key Principles

#### 1. Parent-Child Relationships

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

#### 2. Window-Based Search

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

#### 3. Preferred Direction

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

#### 4. Message Filtering

Rules can filter which messages are eligible for association:

- **`valid_source_messages()`**: Only process source traces with these message names
- **`valid_target_messages()`**: Only consider targets with these message names

This prevents false matches between unrelated message types.

#### 5. Relationship Direction (`source_is_child`)

Each rule defines whether the source trace is the child or parent:

| `source_is_child` | Source Role | Target Role | Example |
|-------------------|-------------|-------------|---------|
| `true` | Child | Parent | NAS finds its RRC carrier |
| `false` | Parent | Child | NGAP finds the NAS it carries |

### Architecture

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

### AssociationRule Trait

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

### Computation Flow

#### compute_associations()

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

#### rules_for_layer()

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

#### Lookback Window

When new batches arrive, traces near the batch boundary are re-processed to catch associations that span batches:

```rust
let lookback_window = 20;
let lookback_start = start_index.saturating_sub(lookback_window);
// Reset NotFound status in lookback range to NotComputed
// Then recompute from lookback_start
```

### Chronograph Integration

Related traces are visually highlighted:
- **Current trace**: Bright blue
- **Parent/child traces**: Lighter blue
- **Other traces**: Theme-aware default color

### Implemented Rules

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


### Files Structure

**New Files:**
- `tramex-tools/src/interface/association/mod.rs` - `compute_associations()`
- `tramex-tools/src/interface/association/relation.rs` - `TraceRelation`, `AssociationStatus`
- `tramex-tools/src/interface/association/rules.rs` - `AssociationRule` trait, implementations
- `tramex-tools/src/interface/association/matcher.rs` - `TraceMatcher`


#### Note on Rule Direction

| Rule | Direction |
|------|-----------|
| `NgapToNasRule` | `source_is_child = false` |
| `NasToRrcRule` | `source_is_child = true` |

There is no particular reason for this difference. Both orders are valid. The variation was primarily for testing the full logic.

**Performance consideration:** There is no difference in performance between the two approaches. However, `source_is_child = false` could be slightly faster if the hex parsing is reused across matches instead of being parsed every time.

---
