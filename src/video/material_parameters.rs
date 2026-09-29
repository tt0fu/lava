use glam::{Vec3, Vec4};
use serde::Deserialize;

use crate::video::{parameters::TypedParameters, shaders};

#[derive(Deserialize)]
pub struct PatternParameters {
    pub lightness: f32,
    pub chroma: f32,
    pub scale: f32,
    pub repeats: f32,
    pub warp_speed: f32,
    pub scroll_speed: f32,
}

impl TypedParameters for PatternParameters {
    type Content = shaders::PatternParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            lightness: self.lightness.into(),
            chroma: self.chroma.into(),
            scale: self.scale.into(),
            repeats: self.repeats.into(),
            warp_speed: self.warp_speed.into(),
            scroll_speed: self.scroll_speed.into(),
        }
    }
}

#[derive(Deserialize)]
pub struct GridnodeParameters {
    pub lightness: f32,
    pub chroma: f32,
}

impl TypedParameters for GridnodeParameters {
    type Content = shaders::GridnodeParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            lightness: self.lightness.into(),
            chroma: self.chroma.into(),
        }
    }
}

#[derive(Deserialize)]
pub struct HnodeParameters {
    pub lightness: f32,
    pub chroma: f32,
}

impl TypedParameters for HnodeParameters {
    type Content = shaders::HnodeParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            lightness: self.lightness.into(),
            chroma: self.chroma.into(),
        }
    }
}

#[derive(Deserialize)]
pub struct ColorParameters {
    pub color: Vec4,
}

impl TypedParameters for ColorParameters {
    type Content = shaders::ColorParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            color: self.color.into(),
        }
    }
}

#[derive(Deserialize)]
pub struct ClockParameters {
    pub col: Vec3,
    pub speed: f32,
}

impl TypedParameters for ClockParameters {
    type Content = shaders::ClockParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            col: self.col.into(),
            speed: self.speed,
        }
    }
}

#[derive(Deserialize)]
pub struct WaveformParameters {
    pub background: Vec4,
    pub foreground: Vec4,
    pub line_width: f32,
    pub gain: f32,
}

impl TypedParameters for WaveformParameters {
    type Content = shaders::WaveformParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            background: self.background.into(),
            foreground: self.foreground.into(),
            line_width: self.line_width,
            gain: self.gain,
        }
    }
}

#[derive(Deserialize)]
pub struct SpectrogramParameters {
    pub background: Vec4,
    pub foreground: Vec4,
    pub min_frequency: f32,
    pub max_frequency: f32,
    pub gain: f32,
    pub add: f32,
    pub circular: bool,
    pub debug: bool,
}

impl TypedParameters for SpectrogramParameters {
    type Content = shaders::SpectrogramParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            background: self.background.into(),
            foreground: self.foreground.into(),
            min_frequency: self.min_frequency,
            max_frequency: self.max_frequency,
            gain: self.gain,
            add: self.add,
            circular: self.circular as u32,
            debug: self.debug as u32,
        }
    }
}

#[derive(Deserialize)]
pub struct BandsParameters {
    pub col: Vec3,
    pub gain: Vec4,
}

impl TypedParameters for BandsParameters {
    type Content = shaders::BandsParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            col: self.col.to_array().into(),
            gain: self.gain.into(),
        }
    }
}
