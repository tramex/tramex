# User Guide

This section is meant to help get started with Tramex.

## Tramex in a nutshell

Tramex is a 4G network scanner. With Tramex, it is possible to analyze each message of a 4G communication. Those messages are called frames. Tramex allows users to read 4G frames and display the content data such as the channels followed during the communication or the RRC state. Tramex was developped to be accessible from several platforms:

- in browser mode: the user only needs to go on the [`tramex website`](https://tramex.github.io/tramex/)
- in binary mode: the user needs to download the binary file of tramex from the [releases page](https://github.com/tramex/tramex/releases) and run it
- in crates mode: the user can download and install Tramex with the command: `cargo install tramex` and then run `tramex` in the terminal

## Get started with Tramex

The Tramex interface is always alike, no matter the mode used. In this section, the various functionalities will be described.

### Documentation access

**General informations** are accessible with a click on the `Menu` button on the left of the top bar then `About`.

**Tramex repository** is also accessible from the Tramex interface with a click on the `About` button on the left of the top bar then [`Tramex repository`](https://github.com/tramex/tramex).

From the Tramex interface, it is possible to access differents documentations:

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

The connection might take a few moments and if Tramex cannot connect to the server using the websocket, an error message should appear in the `Errors` panel. From this window, it is possible to read the description of the error, copy it and [report it at the Tramex repository](https://github.com/tramex/tramex/issues) if necessary.

## Analyze frames with Tramex

Regardless of the chosen mode, the display of the frame analysis using Tramex is always the same. Several windows called panels are available and each of them provide specific information.

This section will define the various panels available.

### Current message panel

The current message panel contains the information regarding the current frame. The `Received events` field points out the total number of messages for the current communication. The `Current msg index` field indicates the frame ID within the analyzed communication. The `MessageType` field informs the user about the date and time when the message was received, the model layer such as RRC or NAS, the direction of the link, the channel used and the message type. Finally, the content of the message il also displayed.

### Logical channels panel

In this panel, it is possible to visualize the channels used to transmit the current message. The panel is divided into three rows and two columns. The first row corresponds to the logical channels, the second row to the transport channels and the thrid row to the physical channels. The three rows are separated into two, the left hand-side corresponds to a downlink communication whereas the right hand-side corresponds to an uplink communication.

This panel allows the user to easily spot the channels followed for the current message by illuminating the name of the used channels with specific colors. Moreover, the direction of the communication is therefore also highlighted.

The color applied to the name of the channel specifies the channel type:

- red is for broadcast channels: the message is intended for every UE in the network
- blue is for shared channels: the message is intended for a group of UEs
- orange is for dedicated channels: the message is intended for a specific UE alone
- green is for trafic channels: the message contains data

### RRC state panel

The RRC state panel contains the information regarding the current RRC state. Initially, the UE is in idle state until the reception of a `RRC Connection Setup` message. The UE then switches to a connected state. Similarly, the UE returns to the idle state when a `RRC Connection Release` message is received.

### Proceed through the frames

To browse through the frames, Tramex implemented two buttons located in the top right corner of the screen. With the `Next` button, it is possible to visualize the following message whereas the `Previous` button allows the user to return to the previous message.

As a communication coveys a lot of information, Tramex enables the user to choose the data to display. By clicking on the `Windows` button in the top right corner, it is possible to select the panels to display. If a panel is closed unintentionally, the user can reopen it with the `Windows` button.

Finally, by clicking on `Options` in the vertical bar on the left of the screen, the user can select the message types to display or not and change the size of the messages to display. By default, the size is set to a maximum of `1024` messages.
