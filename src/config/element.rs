use glam::Vec4;
use serde::Deserialize;

use crate::config::{
    blend::BlendConfig,
    material::MaterialConfig,
    transform::TransformConfig,
};

#[derive(Deserialize)]
pub struct PanelConfig {
    pub transform: TransformRef,
    pub material: MaterialRef,
    pub order: u32,
    pub blend: BlendConfig,
}

#[derive(Deserialize)]
pub struct GroupConfig {
    pub transform: TransformRef,
    pub order: u32,
    pub blend: BlendConfig,
    pub background: Vec4,
    pub children: Vec<ElementConfig>,
}

/// A node of the panel tree: either a leaf panel (has `material`) or a group (has `children`).
#[derive(Deserialize)]
#[serde(untagged)]
pub enum ElementConfig {
    Panel(PanelConfig),
    Group(GroupConfig),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum TransformRef {
    Named(String),
    Inline(TransformConfig),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum MaterialRef {
    Named(String),
    Inline(MaterialConfig),
}
