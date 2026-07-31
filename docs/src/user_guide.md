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

The user can choose the processing mode using the buttons `Choose ws or file` on the top of the left vertical panel.

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

Regardless of the chosen mode, the display of the frame analysis using Tramex is always the same. Several windows called panels are available and each of them provide specific information.

This section will define the various panels available.

---

### Navigation Panel

The Navigation panel provides controls for navigating through events and displays statistics about the current trace session.

**Statistics displayed:**
- **Current event index**: The position of the currently focused event (clickable to jump to a specific event)
- **Total loaded events**: Number of events currently loaded in memory
- **Total events in source**: Total number of events available in the file (if known)
- **Timestamp**: The timestamp of the current event
- **Layer**: The protocol layer of the current event (e.g., RRC, NAS, PHY)

**Navigation controls:**
- **Previous / Next buttons**: Navigate sequentially through events
- **Clickable index**: Click on the event index to enter a specific event number and jump directly to it

**WebSocket mode controls:**
- **Pause / Resume buttons**: Control auto-loading of new events from the WebSocket connection

---

### Message Panel

The Message panel displays the detailed content of the currently selected trace.

**Information displayed:**
- **Layer type**: The protocol layer (RRC, NAS, NGAP, PHY, etc.) shown as a heading
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

### Chronograph Panel

The Chronograph panel displays a visual timeline of message exchanges between network elements.

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
- **PUSCH**: `PUSCH harq={} retx={} rv_idx={} crc={OK/KO}`
- **PUCCH**: `PUCCH format={} {ACK/NACK}`

**Filtering:**
- Automatically filters out non-HARQ events (MIB/SIB broadcasts, CSI-only PUCCH)

**Features:**
- Color-coded HARQ processes for easy tracking
- Focused event highlighted with background bar
- Auto-scrolls to centered on focused event
- Track retransmissions via `retx` and `rv_idx` fields

---

## Options and Settings

### Layer Filtering

The **Layers** option in the left panel allows filtering which protocol layers are displayed.

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

Toggle checkboxes to enable/disable display of specific layers.

---

### AI Settings

The AI Settings panel (accessible via **Settings** menu) configures the AI-powered trace explanation feature.

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

To browse through the frames, use the **Previous** and **Next** buttons in the Navigation panel.

**Jump to specific event:**
- Click on the event index number in the Navigation panel
- Enter the desired event number
- Press Enter to jump directly to that event

### Panel management

- Click on **Windows** in the top menu to show/hide panels
- Panels can be resized and repositioned
- Panel states are saved between sessions
