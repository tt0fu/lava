use serde::Deserialize;

#[derive(Deserialize)]
pub struct MaterialConfig {
    pub shader: String,
    pub parameters: serde_json::Value,
}
