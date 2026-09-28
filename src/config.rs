use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use glam::{Vec2, Vec4};
use serde::Deserialize;
use vulkano::pipeline::graphics::color_blend::{AttachmentBlend, BlendFactor, BlendOp};

use crate::{
    audio::audio_settings::AudioSettings,
    video::transform::{Scalar, Transform, Unit, Vector, anchor},
};

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
    pub background_color: Vec4,

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
    pub anchor: AnchorConfig,
    pub position: VectorConfig,
    pub scale: VectorConfig,
    pub rotation: f32,
}

impl TransformConfig {
    pub fn to_transform(&self) -> Result<Transform> {
        let anchor_type = match &self.anchor {
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
            anchor_type,
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

#[derive(Deserialize)]
pub struct MaterialConfig {
    pub shader: String,
    pub parameters: serde_json::Value,
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
    #[serde(default, alias = "background")]
    pub background_color: Vec4,
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
pub enum BlendConfig {
    Named(String),
    Custom(CustomBlendConfig),
}

impl BlendConfig {
    pub fn to_blend(&self) -> Result<AttachmentBlend> {
        match self {
            BlendConfig::Named(name) => named_blend(name),
            BlendConfig::Custom(custom) => custom.to_blend(),
        }
    }
}

#[derive(Deserialize)]
pub struct CustomBlendConfig {
    pub src_factor: String,
    pub dst_factor: String,
    pub operation: String,
    pub src_alpha_factor: Option<String>,
    pub dst_alpha_factor: Option<String>,
    pub alpha_operation: Option<String>,
}

impl CustomBlendConfig {
    fn to_blend(&self) -> Result<AttachmentBlend> {
        let src_color = parse_blend_factor(&self.src_factor)?;
        let dst_color = parse_blend_factor(&self.dst_factor)?;
        let color_op = parse_blend_op(&self.operation)?;
        let src_alpha = optional(
            self.src_alpha_factor.as_deref(),
            parse_blend_factor,
            src_color,
        )?;
        let dst_alpha = optional(
            self.dst_alpha_factor.as_deref(),
            parse_blend_factor,
            dst_color,
        )?;
        let alpha_op = optional(self.alpha_operation.as_deref(), parse_blend_op, color_op)?;
        Ok(blend(
            src_color, dst_color, color_op, src_alpha, dst_alpha, alpha_op,
        ))
    }
}

fn optional<T: Copy>(value: Option<&str>, parse: fn(&str) -> Result<T>, default: T) -> Result<T> {
    match value {
        Some(value) => parse(value),
        None => Ok(default),
    }
}

fn named_blend(name: &str) -> Result<AttachmentBlend> {
    const REPLACE: AttachmentBlend = blend(
        BlendFactor::One,
        BlendFactor::Zero,
        BlendOp::Add,
        BlendFactor::One,
        BlendFactor::Zero,
        BlendOp::Add,
    );
    // Straight-alpha "over": color is weighted by the source alpha and the destination alpha
    // accumulates `src_a + dst_a * (1 - src_a)`.
    const OVER: AttachmentBlend = blend(
        BlendFactor::SrcAlpha,
        BlendFactor::OneMinusSrcAlpha,
        BlendOp::Add,
        BlendFactor::One,
        BlendFactor::OneMinusSrcAlpha,
        BlendOp::Add,
    );
    const MULTIPLY: AttachmentBlend = blend(
        BlendFactor::DstColor,
        BlendFactor::Zero,
        BlendOp::Add,
        BlendFactor::DstAlpha,
        BlendFactor::Zero,
        BlendOp::Add,
    );
    const MIN: AttachmentBlend = blend(
        BlendFactor::One,
        BlendFactor::One,
        BlendOp::Min,
        BlendFactor::One,
        BlendFactor::One,
        BlendOp::Min,
    );
    const MAX: AttachmentBlend = blend(
        BlendFactor::One,
        BlendFactor::One,
        BlendOp::Max,
        BlendFactor::One,
        BlendFactor::One,
        BlendOp::Max,
    );
    const SUBTRACT: AttachmentBlend = blend(
        BlendFactor::One,
        BlendFactor::One,
        BlendOp::ReverseSubtract,
        BlendFactor::One,
        BlendFactor::One,
        BlendOp::ReverseSubtract,
    );

    Ok(match name {
        "replace" => REPLACE,
        "normal" => OVER,
        "add" => AttachmentBlend::additive(),
        "ignore" => AttachmentBlend::ignore_source(),
        "multiply" => MULTIPLY,
        "darken" => MIN,
        "lighten" => MAX,
        "subtract" => SUBTRACT,
        _ => bail!("unknown blend mode: '{name}'"),
    })
}

const fn blend(
    src_color: BlendFactor,
    dst_color: BlendFactor,
    color_op: BlendOp,
    src_alpha: BlendFactor,
    dst_alpha: BlendFactor,
    alpha_op: BlendOp,
) -> AttachmentBlend {
    AttachmentBlend {
        src_color_blend_factor: src_color,
        dst_color_blend_factor: dst_color,
        color_blend_op: color_op,
        src_alpha_blend_factor: src_alpha,
        dst_alpha_blend_factor: dst_alpha,
        alpha_blend_op: alpha_op,
    }
}

fn parse_blend_factor(name: &str) -> Result<BlendFactor> {
    Ok(match name {
        "zero" => BlendFactor::Zero,
        "one" => BlendFactor::One,
        "src_color" => BlendFactor::SrcColor,
        "one_minus_src_color" => BlendFactor::OneMinusSrcColor,
        "dst_color" => BlendFactor::DstColor,
        "one_minus_dst_color" => BlendFactor::OneMinusDstColor,
        "src_alpha" => BlendFactor::SrcAlpha,
        "one_minus_src_alpha" => BlendFactor::OneMinusSrcAlpha,
        "dst_alpha" => BlendFactor::DstAlpha,
        "one_minus_dst_alpha" => BlendFactor::OneMinusDstAlpha,
        "constant_color" => BlendFactor::ConstantColor,
        "one_minus_constant_color" => BlendFactor::OneMinusConstantColor,
        "constant_alpha" => BlendFactor::ConstantAlpha,
        "one_minus_constant_alpha" => BlendFactor::OneMinusConstantAlpha,
        "src_alpha_saturate" => BlendFactor::SrcAlphaSaturate,
        "src1_color" => BlendFactor::Src1Color,
        "one_minus_src1_color" => BlendFactor::OneMinusSrc1Color,
        "src1_alpha" => BlendFactor::Src1Alpha,
        "one_minus_src1_alpha" => BlendFactor::OneMinusSrc1Alpha,
        _ => bail!("unknown blend factor: '{name}'"),
    })
}

fn parse_blend_op(name: &str) -> Result<BlendOp> {
    Ok(match name {
        "add" => BlendOp::Add,
        "subtract" => BlendOp::Subtract,
        "reverse_subtract" => BlendOp::ReverseSubtract,
        "min" => BlendOp::Min,
        "max" => BlendOp::Max,
        _ => bail!("unknown blend operation: '{name}'"),
    })
}
