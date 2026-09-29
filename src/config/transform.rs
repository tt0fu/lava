use anyhow::{Result, bail};
use glam::Vec2;
use serde::Deserialize;

use crate::video::transform::{Scalar, Transform, Unit, Vector, anchor};

#[derive(Deserialize)]
pub struct TransformConfig {
    pub anchor: AnchorConfig,
    pub position: VectorConfig,
    pub scale: VectorConfig,
    pub rotation: f32,
}

impl TransformConfig {
    pub fn to_transform(&self) -> Result<Transform> {
        let anchor_point = match &self.anchor {
            AnchorConfig::Named(name) => match name.as_str() {
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
            },
            AnchorConfig::Custom(value) => *value,
        };
        Ok(Transform {
            anchor_point,
            anchor_position: self.position.clone().into(),
            scale: self.scale.clone().into(),
            rotation: self.rotation,
        })
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum AnchorConfig {
    Named(String),
    Custom(Vec2),
}

#[derive(Clone, Copy, Deserialize)]
pub struct VectorConfig(pub [(f32, Unit); 2]);

impl From<VectorConfig> for Vector {
    fn from(config: VectorConfig) -> Self {
        let [(x, x_unit), (y, y_unit)] = config.0;
        Vector {
            x: Scalar {
                value: x,
                unit: x_unit,
            },
            y: Scalar {
                value: y,
                unit: y_unit,
            },
        }
    }
}
