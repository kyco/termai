use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Usage {
    #[allow(dead_code)]
    pub input_tokens: u32,
    #[allow(dead_code)]
    pub output_tokens: u32,
    #[serde(default)]
    #[allow(dead_code)]
    pub cache_creation_input_tokens: u32,
    #[serde(default)]
    #[allow(dead_code)]
    pub cache_read_input_tokens: u32,
}
