# User Guide

This section is meant to help get started with Tramex.

## Tramex in a nutshell

Tramex is a 4G/5G network analyzer. With Tramex, it is possible to analyze each message of a 4G (LTE) or 5G (NR) communication. Those messages are called traces. Tramex allows users to read protocol traces and display content data such as the channels followed during the communication, the RRC state, resource block allocations, and HARQ processes. Tramex was developed to be accessible from several platforms:

- **Browser mode**: the user only needs to go on the [`tramex website`](https://tramex.github.io/tramex/)
- **Binary mode**: the user needs to download the binary file of tramex from the [releases page](https://github.com/tramex/tramex/releases) and run it
- **Crates mode**: the user can download and install Tramex with the command: `cargo install tramex` and then run `tramex` in the terminal

## Get started with Tramex

The Tramex interface is always alike, no matter the mode used. In this section, the various functionalities will be described.

### Documentation access

**General information** is accessible with a click on the `Menu` button on the left of the top bar then `About`.

**Tramex repository** is also accessible from the Tramex interface with a click on the `About` button on the left of the top bar then [`Tramex repository`](https://github.com/tramex/tramex).

From the Tramex interface, it is possible to access different documentations:

- **User documentation**: Click on the `About` button on the left of the top bar then [`User documentation`](https://tramex.github.io/tramex/docs/documentation.html).
- **Tramex code documentation**: Click on the `About` button on the left of the top bar then [`tramex type`](https://tramex.github.io/tramex/crates/tramex/).
- **Tramex tools code documentation**: Click on the `About` button on the left of the top bar then [`tramex-tools types`](https://tramex.github.io/tramex/crates/tramex_tools/).

---

### Processing modes

In order to analyze frames, Tramex offers two processing modes:

- File reader: read recorded data from a `.log` file.
- Websocket: read real-time data from a server.

The user can choose the processing mode using the buttons `Choose ws or file` on the top of the left vertical panel. This panel collapses automatically once a file is loaded or a websocket is connected; use the `◀` / `▶` button on the left edge to show or hide it at any time.

#### File reader mode

When using the file reader mode, the user is asked to provide the file to read which can be done in two ways:

- With the file explorer by clicking on the `Open file...` button in the left vertical panel.
- From the files list in the left vertical panel by clicking on the arrow on the left of each directory name until reaching the desired file.

When done with the file, it is possible to close it by clicking on the `Close` button on the left vertical panel. It is necessary to close the current file to open another one or switch to the websocket mode.

#### Websocket mode

When using the websocket mode, the user is asked to provide the IP address as well as the port number of the distant server. The information must be provided in the format `ws://127.0.0.1:9001` where `127.0.0.1` is the example IP address and `9001` is the example port number in the typebar below the processing mode choosing buttons. Then click on the `Connect` button.

The connection might take a few moments and if Tramex cannot connect to the server using the websocket, an error message should appear in the `Errors` panel. From this window, it is possible to read the description of the error, copy it and [report it to the tramex repository](https://github.com/tramex/tramex/issues) if necessary.

---

## Analyze frames with Tramex

Regardless of the chosen mode, the display of the frame analysis using Tramex is always the same. The layout is made of fixed areas (navigation toolbar, Messages panel, status bar) and of floating windows called panels, each of them providing specific information.

This section will define the layout and the various panels available.

---

### Layout overview

```
┌ Menu bar ─────────────────────────────────────────────────────────┐
├ Toolbar: ◀ Previous  ▶ Next  | Event 12 / 4567 | Time 12:03:01.234 │
│                                             [ RRC ] [ NAS ] [ Radio ] │
├────────┬──────────────────────────────────────────────────────────┤
│ Source │  Messages (always visible)                               │
│        │                     + floating panels of the active group │
├────────┴──────────────────────────────────────────────────────────┤
└ Status bar: source | Loaded N / total | auto-loading | Fully loaded ┘
```

---

### Navigation Toolbar

The navigation toolbar is fixed at the top of the main area and is always visible once a source is active.

**Navigation controls:**
- **◀ Previous / ▶ Next buttons**: Navigate sequentially through events. `Next` is shown as `Start` before the first event is selected.
- **Clickable index**: Click on the event index (`Event 12 / 4567`) to type an event number and jump directly to it
- **Time**: The timestamp of the current event

**WebSocket mode controls:**
- **⏸ Pause / ▶ Resume**: Control auto-loading of new events from the WebSocket connection

**Panel groups** (right side of the toolbar): the `RRC`, `NAS` and `Radio` buttons open a set of panels in one click, see [Panel groups](#panel-groups).

---

### Status Bar

The status bar at the bottom of the window shows:
- The active source (file path or websocket URL)
- **Loaded**: number of events in memory, with the total number of events in the file when known
- The auto-loading state in websocket mode (`● auto-loading` / `● paused`)
- `Fully loaded` once the whole source has been read

---

### Panel groups

The analysis panels are floating windows grouped by topic. Selecting a group with the toolbar buttons opens its panels, closes the panels of the other groups and enables the matching protocol layers:

| Group | Panels | Layers enabled |
|-------|--------|----------------|
| **RRC** | RAN Config, RRC Status, Logical Channels | RRC |
| **NAS** | Chronograph RAN, Identity | RRC, NAS, NGAP, GTPU |
| **Radio** | Resource Blocks, HARQ, Chronograph MAC-RLC-PDCP, Power | PHY |

- The **RRC** group is selected automatically when a file is opened or a websocket is connected.
- Clicking the active group closes its panels.
- To mix panels from several groups, or to show a single panel, use the **Windows** menu in the top bar: `Show all` / `Hide all` and one checkbox per panel. The menu stays open while you tick panels; click outside to close it.
- The layer selection can be refined afterwards in **Settings → Layers**.

---

### Message Panel

The Message panel is fixed in the center of the window and displays the detailed content of the currently selected trace.

**Information displayed:**
- **Layer badge**: The protocol layer (RRC, NAS, NGAP, PHY, etc.) shown as a colored badge
- **Additional information**: Direction (UL/DL), channel type, and message name

**Features:**
- **Show full message checkbox**: Toggle to display the raw text content of the message
- **Copy to clipboard button**: When the full message is shown, copy the entire message text to clipboard

**AI Explain feature** (when enabled):
- **AI Explain button**: Sends the current trace to a configured AI provider (e.g., Mistral) for an explanation
- The AI returns a human-readable explanation of the protocol message, displayed as formatted Markdown
- Useful for understanding complex RRC, NAS, or NGAP messages without consulting external documentation

---

### RRC Status Panel

The RRC Status panel visualizes the Radio Resource Control (RRC) connection state machine.

**State machine visualization:**
- Displays the current RRC state: **IDLE**, **INACTIVE** (5G only), or **CONNECTED**
- Shows the direction of the current RRC message with arrows between **BASE STATION** and **USER EQUIPMENT**
- Highlights the current state with distinct colors

**State transitions:**
- **IDLE → CONNECTED**: Triggered by RRC Connection Setup Complete (LTE) or RRC Setup Complete (NR)
- **CONNECTED → IDLE**: Triggered by RRC Connection Release (LTE) or RRC Release (NR)
- **CONNECTED ↔ INACTIVE**: 5G NR specific transitions via RRC Suspend/Resume messages

**Technology awareness:**
- Automatically detects LTE or NR technology and applies the correct state transition rules
- Maintains state history for accurate state reconstruction when navigating backward through events

---

### Logical Channels Panel

The Logical Channels panel visualizes the mapping between logical, transport, and physical channels for the current message.

**Grid layout:**
- **Rows**: Logical channels (top), Transport channels (middle), Physical channels (bottom)
- **Columns**: Downlink (left) and Uplink (right)

**Channel highlighting:**
When an RRC message is focused, the panel highlights the channels that carried that message:

| Color | Channel Type | Description |
|-------|--------------|-------------|
| Red | Broadcast | Message intended for all UEs in the network |
| Blue | Common/Shared | Message intended for a group of UEs |
| Orange | Dedicated | Message intended for a specific UE |
| Green | Traffic | Data channel |

**Technology indicator:**
- Displays the detected technology (LTE or NR) at the top of the panel

**Hover tooltips:**
- Hover over any channel label to see a description of its type and purpose

---

### RAN Config Panel (BST Config)

The RAN Config panel displays parsed configuration fields from RRC System Information Blocks (SIBs) and NAS messages.

**Sections displayed:**

**General:**
- Technology (LTE/NR)
- PCI (Physical Cell Identity)
- Mode (FDD/TDD)
- ARFCN (Absolute Radio Frequency Channel Number)
- I/O Mode (MIMO/SISO)

**SIB1 fields** (when SIB1 message is encountered):
- MCC/MNC (Mobile Country/Network Code)
- Cell ID
- Tracking Area Code
- q-RxLevMin, q-QualMin (cell selection parameters)
- Frequency band, subcarrier spacing, carrier bandwidth (5G NR)

**SIB2/SIB3 fields** (cell reselection parameters):
- q-Hyst (hysteresis)
- s-IntraSearch, t-ReselectionNR
- Neighbor cell lists

**NSSAI section** (5G NR only):
- Displays configured and allowed S-NSSAI (Network Slice Selection Assistance Information)
- Shows SST (Slice/Service Type) with descriptions: eMBB, URLLC, mMTC, V2X
- Indicates which slices are allowed for the UE

**Behavior:**
- Fields persist until a new SIB message updates them
- Supports both 4G LTE and 5G NR message formats

---

### Identity Panel

The Identity panel displays parsed identity information from NAS messages.

**4G LTE fields** (from Attach Accept):
- **PDN Address**: PDN type, IPv4/IPv6 addresses
- **Network**: APN (Access Point Name), DNS servers
- **GUTI (MME)**: MCC, MNC, MME Group ID, MME Code, M-TMSI
- **TAI List**: Tracking Area Codes with decoded MCC/MNC

**5G NR fields** (from Registration Accept / PDU Session Establishment Accept):
- **PDU Address**: Session type, IPv4/IPv6 addresses
- **Network**: DNN (Data Network Name), DNS servers
- **5G-GUTI**: MCC, MNC, AMF Region ID, AMF Set ID, AMF Pointer, 5G-TMSI
- **TAI List**: Tracking Area Codes
- **QoS Rules**: Rule identifier, DQR, precedence, QFI
- **Session AMBR**: Downlink/Uplink maximum bit rates
- **5QI**: 5G QoS Identifier

**Behavior:**
- Automatically switches between 4G and 5G display based on detected technology
- Values persist until a new NAS message updates them

---

### Chronograph RAN Panel

The Chronograph RAN panel displays a visual timeline of message exchanges between network elements.

**Layout:**
- Three vertical lifelines: **UE** (User Equipment), **BST** (Base Station), **CN** (Core Network)
- Horizontal arrows represent messages flowing between elements
- Messages are displayed chronologically from top to bottom

**Arrow mapping:**
| Layer | Direction | Arrow |
|-------|-----------|-------|
| RRC | UL | UE → BST |
| RRC | DL | BST → UE |
| NAS | UL | UE → CN |
| NAS | DL | CN → UE |
| NGAP | TO | BST → CN |
| NGAP | FROM | CN → BST |
| GTPU | TO | BST → CN |
| GTPU | FROM | CN → BST |

**Highlighting:**
- **Current event**: Bright blue, thicker arrow
- **Related events** (parent/child traces): Lighter blue
- **Other events**: Theme-aware default color

**Features:**
- Auto-scrolls to center on the focused event
- Displays layer type and message name on each arrow
- Click an arrow to navigate to the corresponding trace

---

### Chronograph MAC-RLC-PDCP Panel

This panel follows the same timeline, highlighting, auto-scroll, and click-to-navigate principles as the [Chronograph RAN Panel](#chronograph-ran-panel). Its key differences are:

- Two lifelines only: **UE** and **BST**
- Displays **MAC**, **RLC**, and **PDCP** traces
- **UL** arrows flow from UE to BST; **DL** arrows flow from BST to UE
- Arrow labels contain the layer and full log message; hover an arrow to read the full message in a tooltip

Refer to the [Chronograph RAN Panel](#chronograph-ran-panel) for the common chronograph behavior.

---

### Resource Blocks Panel

The Resource Blocks panel visualizes the 5G NR Physical Resource Block (PRB) allocations over time.

**Grid axes:**
- **Y-axis (Frequency)**: Physical Resource Blocks (PRBs), typically 51 PRBs
- **X-axis (Time)**: Symbols (detailed) or Slots (aggregated)

**View modes:**
- **Symbol view**: 14 symbols per slot, detailed allocation visualization
- **Slot view**: Aggregated view showing highest-priority resource per PRB per slot

**Resource types and colors:**
| Type | Color | Description |
|------|-------|-------------|
| PDCCH | Dark green | Downlink control channel |
| PUCCH | Light green | Uplink control channel |
| PDSCH | Blue | Downlink data channel |
| PUSCH | Cyan | Uplink data channel |
| PRACH | Yellow | Random access channel |
| SSB | Magenta | Synchronization Signal Block |
| DMRS | Red | Demodulation Reference Signal |

**Navigation:**
- **Frame navigation**: ◀◀ / ▶▶ buttons to move by frames (10ms)
- **Subframe navigation**: ◀ / ▶ buttons to move by subframes (1ms)
- **Cell size slider**: Adjust the size of grid cells for visibility

**Features:**
- Auto-centers on focused PHY event
- Highlights cells belonging to the focused event
- Displays frame range at the top
- Hover over cells for detailed PHY allocation info
- Grays out slots outside the received event range

---

### HARQ Panel

The HARQ panel provides a chronograph-style visualization of PHY layer events, showing the HARQ (Hybrid Automatic Repeat Request) process flow.

**Layout:**
- Two vertical lifelines: **UE** and **BST**
- Arrows colored by HARQ process number (0-15)
- Legend shows active HARQ processes with their colors

**Supported channels:**
| Channel | Direction | Role |
|---------|-----------|------|
| PDCCH | DL | Control channel carrying DCI grants |
| PDSCH | DL | Downlink data |
| PUSCH | UL | Uplink data |
| PUCCH | UL | Uplink control (ACK/NACK feedback) |

**Arrow labels:**
- **PDCCH**: `PDCCH dci={format} ndi={} rv_idx={}`
- **PDSCH**: `PDSCH harq={} retx={} rv_idx={}`
- **PUSCH**: `PUSCH harq={} retx={} rv_idx={} crc={OK/KO} {ACK/NACK}`
- **PUCCH**: `PUCCH format={} {ACK/NACK}`

**Filtering:**
- Automatically filters out non-HARQ events (MIB/SIB broadcasts, CSI-only PUCCH)
- It is possible to filter by HARQ process to better understand the sequence

**Features:**
- Color-coded HARQ processes for easy tracking
- Focused event highlighted with background bar
- Click an arrow to navigate to the corresponding trace
- Auto-scrolls to centered on focused event
- Track retransmissions via `retx` and `rv_idx` fields

---

### Power Panel

The Power panel displays three point graphs sharing a common SFN (System Frame Number) X axis, built from uplink measurements reported on **PUSCH** and **PUCCH** PHY traces:

| Metric | Description |
|--------|-------------|
| **EPRE** | Energy Per Resource Element (dB) |
| **TA** | Timing Advance (µs) |
| **CSI** | Channel State Information (decoded from binary, e.g. `0101` → 5) |

**Layout:**
- Three stacked graphs, each with its own Y axis auto-scaled to visible data
- Shared X axis labeled with frame numbers at frame boundaries
- Points only (no lines) since events do not arrive in SFN order

**Controls:**
- **Window slider**: Adjust the number of frames displayed (5–500)
- **◀◀ / ▶▶**: Navigate by one frame
- **PUSCH / PUCCH toggles**: Show or hide samples from each channel type
- **Drag**: Pan the X window left/right

**Features:**
- Focused event highlighted with an accent ring and a vertical marker across all graphs
- Hover over a point to see all measurements in a tooltip (including raw CSI bits)
- Click a point to navigate to the corresponding trace
- Window automatically re-centers on the focused event

---

## Options and Settings

All options are grouped in the **Settings** window, opened from the **Settings** menu in the top bar. The window has three tabs: **AI**, **Layers** and **File options**.

### Layer Filtering

The **Layers** tab allows filtering which protocol layers are displayed. Selecting a panel group in the toolbar applies a preset (see [Panel groups](#panel-groups)); this tab lets you adjust it.

**Radio layers:**
- PHY (Physical)
- MAC (Medium Access Control)
- RLC (Radio Link Control)
- PDCP (Packet Data Convergence Protocol)
- SDAP (Service Data Adaptation Protocol) - 5G only
- RRC (Radio Resource Control)
- NAS (Non-Access Stratum)

**Core Network layers:**
- S72, S1AP (4G LTE)
- NGAP (5G NR)
- GTPU (User plane tunneling)
- X2AP, XnAP (inter-eNB/gNB interfaces)
- M2AP, LPPa, NRPPa (positioning)
- TRX (transceiver)

Toggle checkboxes to enable/disable display of specific layers. In file mode, disabled layers are skipped when navigating with `Previous` / `Next`; in websocket mode, only the enabled layers are requested from the server.

---

### File options

The **File options** tab (file mode only) sets the URL of the index of files used to populate the files list in the left panel, with a `Reload list` button.

---

### AI Settings

The **AI** tab configures the AI-powered trace explanation feature.

**Configuration options:**
- **Provider**: Select AI provider (e.g., Mistral)
- **Model**: Select specific model (e.g., mistral-medium-latest)
- **API Key**: Enter your API key directly or load from environment variable
- **Environment variable**: Configure the variable name (default: `TRAMEX_AI_API_KEY`)

**Usage:**
1. Set `TRAMEX_AI_API_KEY` environment variable before running Tramex, or enter the key in settings
2. Navigate to any trace in the Message panel
3. Click "AI Explain" to get a human-readable explanation

**Security:**
- API keys are not saved to disk (must be re-entered or loaded from environment each session)

---

## Navigation Tips

### Proceed through the frames

To browse through the frames, use the **Previous** and **Next** buttons in the navigation toolbar.

**Jump to specific event:**
- Click on the event index number in the navigation toolbar
- Enter the desired event number
- Press Enter to jump directly to that event

### Panel management

- Use the **RRC** / **NAS** / **Radio** buttons in the toolbar to switch between panel groups
- Click on **Windows** in the top menu to show/hide individual panels
- Panels can be resized and repositioned; `Menu → Organize windows` resets their positions
- Open panels are saved between sessions
- The light/dark theme switch is on the left of the top bar
