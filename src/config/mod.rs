mod blend;
mod element;
mod material;
mod transform;

pub use blend::BlendConfig;
pub use element::{ElementConfig, GroupConfig, MaterialRef, PanelConfig, TransformRef};
pub use material::MaterialConfig;
pub use transform::{AnchorConfig, TransformConfig, VectorConfig};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use glam::Vec4;
use serde::Deserialize;

use crate::audio::audio_settings::AudioSettings;

#[derive(Deserialize)]
pub struct Config {
    pub audio: AudioSettings,
    #[serde(default)]
    pub transforms: HashMap<String, TransformConfig>,
    #[serde(default)]
    pub images: HashMap<String, String>,
    #[serde(default)]
    pub materials: HashMap<String, MaterialConfig>,
    pub panels: ElementConfig,
    #[serde(default)]
    pub background: Vec4,

    #[serde(skip)]
    pub base_dir: PathBuf,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut config: Self = jsonc::read_jsonc_sync(path)?;
        config.base_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        Ok(config)
    }
}
