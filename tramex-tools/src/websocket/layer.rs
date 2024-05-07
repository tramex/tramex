//! Layer enum and Layers struct
use std::str::FromStr;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
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

    /// None layer - is good idea ?
    None,
}

impl Default for Layer {
    fn default() -> Self {
        Layer::None
    }
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

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
/// Layers struct
pub struct Layers {
    #[serde(rename(serialize = "PHY"))]
    /// Physical layer
    pub phy: String,

    #[serde(rename(serialize = "MAC"))]
    /// Medium Access Control layer
    pub mac: String,

    #[serde(rename(serialize = "RLC"))]
    /// Radio Link Control layer
    pub rlc: String,

    #[serde(rename(serialize = "PDCP"))]
    /// Packet Data Convergence Protocol layer
    pub pdcp: String,

    #[serde(rename(serialize = "RRC"))]
    /// Radio Resource Control layer
    pub rrc: String,

    #[serde(rename(serialize = "NAS"))]
    /// Non Access Stratum layer
    pub nas: String,

    #[serde(rename(serialize = "S72"))]
    /// S1 Application Protocol layer
    pub s72: String,

    #[serde(rename(serialize = "S1AP"))]
    /// S1 Application Protocol layer
    pub s1ap: String,

    #[serde(rename(serialize = "NGAP"))]
    /// Next Generation Application Protocol layer
    pub ngap: String,

    #[serde(rename(serialize = "GTPU"))]
    /// GTPU layer
    pub gtpu: String,

    #[serde(rename(serialize = "X2AP"))]
    /// X2 Application Protocol layer
    pub x2ap: String,

    #[serde(rename(serialize = "XnAP"))]
    /// Xn Application Protocol layer
    pub xnap: String,

    #[serde(rename(serialize = "M2AP"))]
    /// M2 Application Protocol layer
    pub m2ap: String,

    #[serde(rename(serialize = "LPPa"))]
    /// LTE Positioning Protocol A layer
    pub lppa: String,

    #[serde(rename(serialize = "NRPPa"))]
    /// NR Positioning Protocol A layer
    pub nrppa: String,

    #[serde(rename(serialize = "TRX"))]
    /// TRX layer
    pub trx: String,
}

impl Layers {
    /// Create new Layers struct
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
