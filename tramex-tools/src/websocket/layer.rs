use std::str::FromStr;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
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
