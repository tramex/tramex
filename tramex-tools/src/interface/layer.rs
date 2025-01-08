//! Layer enum and Layers struct
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
/// Layer enum
pub enum Layer {
    /// Physical layer
    PHY,

    /// Medium Access Control layer
    MAC,

    /// Radio Link Control layer
    RLC,

    /// Packet Data Convergence Protocol layer
    PDCP,

    /// Radio Resource Control layer
    RRC,

    /// Non Access Stratum layer
    NAS,

    /// S1 Application Protocol layer
    S1AP,

    /// Next Generation Application Protocol layer
    NGAP,

    /// X2 Application Protocol layer
    X2AP,

    /// Xn Application Protocol layer
    XNAP,

    /// M2 Application Protocol layer
    M2AP,

    /// LTE Positioning Protocol A layer
    LPPA,

    /// NR Positioning Protocol A layer
    NRPPA,

    /// GTPU layer
    GTPU,

    /// Special layer of amarisoft
    PROD,
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
            _ => Err(()),
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone, Default)]
/// LayerLogLevel enum
pub enum LayerLogLevel {
    /// Debug log level
    Debug,

    #[default]
    /// Warn log level
    Warn,
}

impl serde::Serialize for LayerLogLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            LayerLogLevel::Debug => serializer.serialize_str("debug"),
            LayerLogLevel::Warn => serializer.serialize_str("warn"),
        }
    }
}

impl Display for LayerLogLevel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LayerLogLevel::Debug => write!(f, "debug"),
            LayerLogLevel::Warn => write!(f, "warn"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
/// Layers struct
pub struct Layers {
    #[serde(rename(serialize = "PHY"))]
    /// Physical layer
    pub phy: LayerLogLevel,

    #[serde(rename(serialize = "MAC"))]
    /// Medium Access Control layer
    pub mac: LayerLogLevel,

    #[serde(rename(serialize = "RLC"))]
    /// Radio Link Control layer
    pub rlc: LayerLogLevel,

    #[serde(rename(serialize = "PDCP"))]
    /// Packet Data Convergence Protocol layer
    pub pdcp: LayerLogLevel,

    #[serde(rename(serialize = "RRC"))]
    /// Radio Resource Control layer
    pub rrc: LayerLogLevel,

    #[serde(rename(serialize = "NAS"))]
    /// Non Access Stratum layer
    pub nas: LayerLogLevel,

    #[serde(rename(serialize = "S72"))]
    /// S1 Application Protocol layer
    pub s72: LayerLogLevel,

    #[serde(rename(serialize = "S1AP"))]
    /// S1 Application Protocol layer
    pub s1ap: LayerLogLevel,

    #[serde(rename(serialize = "NGAP"))]
    /// Next Generation Application Protocol layer
    pub ngap: LayerLogLevel,

    #[serde(rename(serialize = "GTPU"))]
    /// GTPU layer
    pub gtpu: LayerLogLevel,

    #[serde(rename(serialize = "X2AP"))]
    /// X2 Application Protocol layer
    pub x2ap: LayerLogLevel,

    #[serde(rename(serialize = "XnAP"))]
    /// Xn Application Protocol layer
    pub xnap: LayerLogLevel,

    #[serde(rename(serialize = "M2AP"))]
    /// M2 Application Protocol layer
    pub m2ap: LayerLogLevel,

    #[serde(rename(serialize = "LPPa"))]
    /// LTE Positioning Protocol A layer
    pub lppa: LayerLogLevel,

    #[serde(rename(serialize = "NRPPa"))]
    /// NR Positioning Protocol A layer
    pub nrppa: LayerLogLevel,

    #[serde(rename(serialize = "TRX"))]
    /// TRX layer
    pub trx: LayerLogLevel,
}

impl Layers {
    /// Create new Layers struct
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new Layers but in an optiniated way
    pub fn new_optiniated() -> Self {
        let mut layers = Layers::new();
        layers.rrc = LayerLogLevel::Debug;
        layers
    }

    /// Create new Layers struct with all debug
    pub fn all_debug() -> Self {
        Self {
            phy: LayerLogLevel::Debug,
            mac: LayerLogLevel::Debug,
            rlc: LayerLogLevel::Debug,
            pdcp: LayerLogLevel::Debug,
            rrc: LayerLogLevel::Debug,
            nas: LayerLogLevel::Debug,
            s72: LayerLogLevel::Debug,
            s1ap: LayerLogLevel::Debug,
            ngap: LayerLogLevel::Debug,
            gtpu: LayerLogLevel::Debug,
            x2ap: LayerLogLevel::Debug,
            xnap: LayerLogLevel::Debug,
            m2ap: LayerLogLevel::Debug,
            lppa: LayerLogLevel::Debug,
            nrppa: LayerLogLevel::Debug,
            trx: LayerLogLevel::Debug,
        }
    }
}
