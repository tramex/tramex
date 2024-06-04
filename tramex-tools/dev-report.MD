# Dev Report - Tramex Tools

## What you can find in this report

    This report is here to explain the choices we made developping this crate, and explaining the architecture and what we had in mind when we designed this crate.

## What is tramex-tools

    Tramex tools is a crate for the Tramex Project.

### Connector
You will find in here the `Connector` (connector.rs), that is the structure that handles reading from different sources to return `Data`. It is the main exposed struct of this crate, and the point of entry when used by the project Tramex.

#### First what is `Data`

`Data` is what the `Connector` aims to retreive, in it is a Vector of `Trace` and the index of the current `Trace`.
A `Trace` is a struct with a `MessageType` and a `Hexa`.
It represents a 4G trace, with in `MessageType` the information about what `Layer` is this message, what is the `direction` of the trace, from which `Canal` does it comes from etc...
All this `MessageType` information, is used to know what type of trace it is, so we can rightly decode the `Hexa`.
The `Hexa` is hexadecimal representation of ASN1 binary, that when rightly decode, can give all information about the content of the trace.

#### What is an `Interface`

Now that we know what we want to retreive, we need to figure out how. 
For this project, we gather the 4G traces from a Software 4G Core, powered by Amarisoft.
We can retreive traces two ways, first by reading a .log File with multiple traces, or we can retreive traces by asking a Websocket API.

So we have two different `Interface`: `File` and `WsConnection`.


#### The Code

In `Connector`, you'll see first all the different constructors, but the two major part of this Structure, is the `get_more_data` and `try_recv` functions, that are what you call when you use a Connector.

`get_more_data` is the function that is called when you want to retreive more `Data` into the `Connector`.
For the `WsConnection` `Interface`, it means sending a WsMessage to the WebSocket server for more traces.
For the `File` `Interface`, it means (if the file is not already entirely read) to try and read the new part of the `File` and save the `Data` in the `Connector`.

#### How does `WsConnection` works

But to dive deeper into how the data is retreived, you need to understand how a `WsConnection` works.
When a `WsConnection` is opened, all the messages sent by the server goes into a receive buffer, and there is typically two ways of handling theses received messages. First you can when you want try to read the buffer to see if something has appeared. Or you can give the WsConnection a callback, to wake up the UI.

#### Back to the Code

The`try_recv` function, is the function that reads from the `WsConnection` Receive buffer, it is typically called everytime the UI is refreshed.

### Parsing

I am going to talk now about details in parsing, when the `WsConnection` or `File` `Interface` serialize its stream

#### `File` Parsing 

the `File` parsing is done in the `file_handler` file.
The entry of the parsing is the 

TODO Finish