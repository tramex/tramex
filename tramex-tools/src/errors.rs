#[derive(serde::Deserialize, Debug)]
pub struct TramexError {
    pub message: String,
    pub code: u32,
    pub recoverable: bool,
}
