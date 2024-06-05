# Amarisoft File Log Format

## LTE Software eNodeB and NR Software gNB

### PHY

#### Format
time layer dir ue_id cell rnti frame.subframe channel:short_content
        long_content

#### Doc
time
Time using the selected format.

layer
Layer ([PHY] here).

dir
UL (uplink) or DL (downlink).

ue_id
eNodeB UE identifier (hexadecimal, unique among all cells).

cell
Low 8 bits of the cell identifier (hexadecimal).

rnti
Associated RNTI (hexadecimal) or - if none.

frame.subframe
Frame number (0-1023) and either subframe number (0-9) for LTE and NB-IoT cells or slot number for NR cells.

channel
PHY channel name (e.g. PUSCH, PUCCH, PRACH, SRS, PSS, PBCH, PCFICH, PDSCH, PHICH, PDCCH, EPDCCH, ...).

short_content
Single line content.

long_content
Hexadecimal dump of the message if phy.max_size > 0.

### RLC PDCP NAS
#### Format

When a message is dumped, the format is:

time layer - ue_id message

When a PDU is dumped (debug level), the format is:

time layer dir ue_id short_content
        long_content
#### Doc
time
Time using the selected format

layer
Layer ([RLC], [PDCP], or [NAS] here).

dir
UL (uplink) or DL (downlink).

ue_id
eNodeB UE identifier (hexadecimal, unique among all cells).

short_content
Single line content.

RLC, PDCP: preceded by the SRB or DRB identifier.
long_content
NAS: full content of the NAS message if layer.max_size > 0.

### MAC RRC

#### Format
When a message is dumped, the format is:

time layer - ue_id message
When a PDU is dumped (debug level), the format is:

time layer dir ue_id short_content
        long_content
#### Doc

time
Time using the selected format

layer
Layer ([MAC] or [RRC] here).

dir
UL (uplink) or DL (downlink).

ue_id
eNodeB UE identifier (hexadecimal, unique among all cells).

cell_id
Primary cell identifier. See cell_id

short_content
Single line content.

long_content
MAC: hexadecimal dump of the message if layer.max_size > 0.
RRC: full ASN.1 content of the RRC message if layer.max_size > 0.
long_content
MAC, RLC, PDCP: hexadecimal dump of the message if layer.max_size > 0.
RRC: full ASN.1 content of the RRC message if layer.max_size > 0.



### S1AP NGAP X2AP XnAP M2AP GTP-U

#### Format
When a message is dumped, the format is:

time layer - message
When a PDU is dumped (debug level), the format is:

time layer dir ip_address short_content
        long_content

#### Doc
time
Time using the selected format.

layer
Layer (e.g. [S1AP]).

dir
Direction: TO or FROM.

ip_address
Source or destination IP address, depending on the dir field.

short_content
Single line content.

long_content
S1AP, NGAP, X2AP, XnAP, M2AP: full ASN.1 content of the message if layer.max_size > 0.
GTPU: hexadecimal dump of the message if layer.max_size > 0.



## LTE and NR Core Network

### NAS
#### Format
When a NAS message is dumped, the format is:

time layer - message
When a NAS data PDU is dumped (debug level), the format is:

time layer dir MME_UE_ID message_type
        long_content
#### Doc
time
Time using the selected format

layer
Indicate the layer ([NAS] here).

dir
UL (uplink) or DL (downlink).

MME_UE_ID
MME S1AP UE identifier (hexadecimal).

message_type
NAS message type.

long_content
Full content of the NAS message if nas.max_size > 0.

### IP
#### Format
When a IP data PDU is dumped (debug level), the format is:

time layer dir short_content
        long_content

#### Doc
time
Time using the selected format

layer
Indicate the layer ([IP] here).

dir
UL (uplink) or DL (downlink).

short_content
Single line content (at least the IP protocol and the source and destination address).

long_content
Optional hexadecimal dump of the PDU if ip.max_size > 0.

### S1AP NGAP SBcAP LCSAP GTP-U 

#### Format
When a message is dumped, the format is:

time layer - message
When a data PDU is dumped (debug level), the format is:

time layer dir ip_address short_content
        long_content

#### Doc
time
Time using the selected format.

layer
Indicate the layer ([S1AP], [NGAP], [SBCAP], [LCSAP], or [GTPU] here).

dir
Direction: TO or FROM.

ip_address
source or destination IP address, depending on the dir field.

short_content
Single line content.

long_content
S1AP, NGAP, SBCAP, LCSAP: full ASN.1 content of the message if layer.max_size > 0.
GTPU: hexadecimal dump of the message if layer.max_size > 0.


## LTE IMS Server

### IMS SIP
#### Format
time layer dir id message
#### Doc
time
Time using the selected format.

layer
Indicate the layer.

dir
FROM or TO or - (No direction associated).

id
For IMS, represents a unique ID associated with a UE binding.
For SIP, represents a unique ID associated to a SIP dialog.

message
Log message.

### CX RX
#### Format
time layer dir addr message

#### Doc
time
Time using the selected format.

layer
Indicate the layer.

dir
FROM or TO or - (No direction associated).

addr
Source IP address for incoming messages.
Destination IP address for outgoing messages.
message
Log message.

### MEDIA
#### Format
time layer id dir protocol/media message

#### Doc
time
Time using the selected format.

layer
Indicate the layer.

dir
FROM or TO or - (No direction associated).

id
SIP associated dialog id.

protocol
Can be either RTP or RTCP.

media
Media type: audio, video or text.

message
Log message.

