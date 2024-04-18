use ewebsock::{WsReceiver, WsSender};
use std::str::FromStr;

// deserialize the message
#[derive(serde::Deserialize, Debug)]
pub struct WebSocketLog {
    pub message: String,         // Same as request
    pub message_id: Option<u64>, //Any type, force as string // Same as in request.
    pub time: f64, // Number representing time in seconds since start of the process. // Usefull to send command with absolute time.
    pub utc: f64,  //Number representing UTC seconds.
    pub logs: Vec<OneLog>,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub enum Layer {
    PHY,
    MAC,
    RLC,
    PDCP,
    RRC,
    NAS,
    S1AP,
    NGAP,
    X2AP,
    XNAP,
    M2AP,
    LPPA,
    NRPPA,
    GTPU,
    None, // is good idea ?
}

impl FromStr for Layer {
    type Err = ();

    fn from_str(input: &str) -> Result<Layer, Self::Err> {
        match input {
            "PHY" => Ok(Layer::PHY),
            "MAC" => Ok(Layer::MAC),
            "RLC" => Ok(Layer::RLC),
            "PDCP" => Ok(Layer::PDCP),
            "RRC" => Ok(Layer::RRC),
            "NAS" => Ok(Layer::NAS),
            "S1AP" => Ok(Layer::S1AP),
            "NGAP" => Ok(Layer::NGAP),
            "X2AP" => Ok(Layer::X2AP),
            "XNAP" => Ok(Layer::XNAP),
            "M2AP" => Ok(Layer::M2AP),
            "LPPA" => Ok(Layer::LPPA),
            "NRPPA" => Ok(Layer::NRPPA),
            "GTPU" => Ok(Layer::GTPU),
            _ => Ok(Layer::None),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LogLevel {
    ERROR = 1,
    WARN = 2,
    INFO = 3,
    DEBUG = 4,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub enum SourceLog {
    ENB,
    MME,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub enum Direction {
    UL,
    DL,
    FROM,
    TO,
}

#[derive(serde::Deserialize, Debug)]
pub struct OneLog {
    pub data: Vec<String>,      // Each item is a string representing a line of log.
    pub timestamp: u64,         // Milliseconds since January 1st 1970.
    pub layer: Layer,           // log layer
    pub level: LogLevel,        // Log level: error, warn, info or debug.
    pub dir: Option<Direction>, //  Log direction: UL, DL, FROM or TO.
    pub cell: Option<u64>,      // cell id
    pub channel: Option<String>, // channel
    pub src: SourceLog,
    pub idx: u64,
}

impl OneLog {
    pub fn extract_hexe(&self) -> Vec<u8> {
        return extract_hexe(&self.data);
    }

    pub fn extract_canal_msg(&self) -> Option<String> {
        if let Some(data_line) = self.data.first() {
            log::info!("{:?}", data_line);
            return Some(data_line.to_owned());
        }
        None
    }
}

impl<'de> serde::Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let a = u8::deserialize(deserializer)?;
        match a {
            1 => Ok(LogLevel::ERROR),
            2 => Ok(LogLevel::WARN),
            3 => Ok(LogLevel::INFO),
            4 => Ok(LogLevel::DEBUG),
            _ => Ok(LogLevel::INFO), // default
        }
    }
}

pub struct WsConnection {
    pub ws_sender: Box<WsSender>,
    pub ws_receiver: Box<WsReceiver>,
    pub connecting: bool,
    pub error_str: Option<String>,
}

impl core::fmt::Debug for WsConnection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Interface")
            .field("ws_sender", &"Box<WsSender>")
            .field("ws_receiver", &"Box<WsReceiver>")
            .field("connecting", &self.connecting)
            .field("error_str", &self.error_str)
            .finish()
    }
}

use serde::{Deserialize, Serialize};

use crate::functions::extract_hexe;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Layers {
    #[serde(rename(serialize = "PHY"))]
    pub phy: String,
    #[serde(rename(serialize = "MAC"))]
    pub mac: String,
    #[serde(rename(serialize = "RLC"))]
    pub rlc: String,
    #[serde(rename(serialize = "PDCP"))]
    pub pdcp: String,
    #[serde(rename(serialize = "RRC"))]
    pub rrc: String,
    #[serde(rename(serialize = "NAS"))]
    pub nas: String,
    #[serde(rename(serialize = "S72"))]
    pub s72: String,
    #[serde(rename(serialize = "S1AP"))]
    pub s1ap: String,
    #[serde(rename(serialize = "NGAP"))]
    pub ngap: String,
    #[serde(rename(serialize = "GTPU"))]
    pub gtpu: String,
    #[serde(rename(serialize = "X2AP"))]
    pub x2ap: String,
    #[serde(rename(serialize = "XnAP"))]
    pub xnap: String,
    #[serde(rename(serialize = "M2AP"))]
    pub m2ap: String,
    #[serde(rename(serialize = "LPPa"))]
    pub lppa: String,
    #[serde(rename(serialize = "NRPPa"))]
    pub nrppa: String,
    #[serde(rename(serialize = "TRX"))]
    pub trx: String,
}
impl Layers {
    pub fn new() -> Self {
        Self {
            phy: "debug".to_owned(),
            mac: "warn".to_owned(),
            rlc: "warn".to_owned(),
            pdcp: "warn".to_owned(),
            rrc: "debug".to_owned(),
            nas: "debug".to_owned(),
            s72: "warn".to_owned(),
            s1ap: "warn".to_owned(),
            ngap: "warn".to_owned(),
            gtpu: "warn".to_owned(),
            x2ap: "warn".to_owned(),
            xnap: "warn".to_owned(),
            m2ap: "warn".to_owned(),
            lppa: "warn".to_owned(),
            nrppa: "warn".to_owned(),
            trx: "warn".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogGet {
    timeout: u64,
    min: u64,
    max: u64,
    layers: Layers,
    message: String,
    headers: bool,
    message_id: u64,
}

impl LogGet {
    pub fn new(id: u64, layers: Layers, max_size: u64) -> Self {
        Self {
            timeout: 1,
            min: 64,
            max: max_size,
            layers: layers,
            message: "log_get".to_owned(),
            headers: false,
            message_id: id,
        }
    }
}
