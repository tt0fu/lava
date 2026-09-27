use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use glam::{Vec2, Vec3};
use serde::Deserialize;

use crate::{
    audio::audio_settings::AudioSettings,
    video::transform::{Transform, Unit, Vector, anchor},
};

#[derive(Deserialize)]
pub struct Config {
    pub audio: AudioSettings,
    pub transforms: HashMap<String, TransformConfig>,
    #[serde(default)]
    pub images: HashMap<String, String>,
    pub materials: HashMap<String, MaterialConfig>,
    pub panels: Vec<PanelConfig>,
    pub background_color: Vec3,

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

#[derive(Deserialize)]
pub struct TransformConfig {
    #[serde(flatten)]
    pub anchor: HashMap<String, VectorConfig>,
    pub scale: VectorConfig,
    pub rotation: f32,
}

impl TransformConfig {
    pub fn to_transform(&self) -> Result<Transform> {
        if self.anchor.len() != 1 {
            bail!(
                "transform must specify exactly one anchor point, found {}",
                self.anchor.len()
            );
        }
        let (name, position) = self.anchor.iter().next().unwrap();
        let anchor_type = match name.as_str() {
            "center" => anchor::CENTER,
            "top" => anchor::TOP,
            "bottom" => anchor::BOTTOM,
            "left" => anchor::LEFT,
            "right" => anchor::RIGHT,
            "top left" => anchor::TOP_LEFT,
            "top right" => anchor::TOP_RIGHT,
            "bottom left" => anchor::BOTTOM_LEFT,
            "bottom right" => anchor::BOTTOM_RIGHT,
            _ => bail!("unknown anchor point: '{name}'"),
        };
        Ok(Transform {
            anchor_type,
            anchor_position: position.clone().into(),
            scale: self.scale.clone().into(),
            rotation: self.rotation,
        })
    }
}

#[derive(Clone, Deserialize)]
pub enum VectorConfig {
    #[serde(rename = "screen")]
    Screen(Vec2),
    #[serde(rename = "pixels")]
    Pixels(Vec2),
}

impl From<VectorConfig> for Vector {
    fn from(config: VectorConfig) -> Self {
        match config {
            VectorConfig::Screen(value) => Vector {
                value,
                unit: Unit::Screen,
            },
            VectorConfig::Pixels(value) => Vector {
                value,
                unit: Unit::Pixels,
            },
        }
    }
}

#[derive(Deserialize)]
pub struct MaterialConfig {
    pub shader: String,
    pub parameters: serde_json::Value,
}

#[derive(Deserialize)]
pub struct PanelConfig {
    pub transform: String,
    pub material: String,
    pub order: u32,
}
