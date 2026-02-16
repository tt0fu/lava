use glam::{Vec3, vec3};
use serde::{Deserialize, Serialize};
use vulkano::padded::Padded;

use crate::video::shaders;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Pattern {
    pub color: Vec3,
    pub use_rainbow: bool,
    pub lightness: f32,
    pub chroma: f32,
    pub scale: f32,
    pub repeats: f32,
    pub pattern_speed: f32,
    pub scroll_speed: f32,
}

impl Default for Pattern {
    fn default() -> Self {
        Self {
            color: vec3(1.0, 1.0, 1.0),
            use_rainbow: true,
            lightness: 0.8,
            chroma: 0.1,
            scale: 1.0,
            repeats: 2.0,
            pattern_speed: 1.0,
            scroll_speed: 2.0,
        }
    }
}

impl From<Pattern> for shaders::Pattern {
    fn from(value: Pattern) -> Self {
        Self {
            color: value.color.to_array(),
            use_rainbow: value.use_rainbow as u32,
            lightness: value.lightness,
            chroma: value.chroma,
            scale: value.scale,
            repeats: value.repeats,
            pattern_speed: value.pattern_speed,
            scroll_speed: value.scroll_speed,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WaveformParameters {
    pub pattern: Pattern,
    pub line_width: f32,
    pub gain: f32,
}

impl Default for WaveformParameters {
    fn default() -> Self {
        Self {
            pattern: Default::default(),
            line_width: 50.0,
            gain: 0.9,
        }
    }
}

impl From<WaveformParameters> for shaders::WaveformParameters {
    fn from(value: WaveformParameters) -> Self {
        Self {
            pattern: Padded(value.pattern.into()),
            line_width: value.line_width,
            gain: value.gain,
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SimplePatternParameters {
    pub pattern: Pattern,
}

impl From<SimplePatternParameters> for shaders::SimplePatternParameters {
    fn from(value: SimplePatternParameters) -> Self {
        Self {
            pattern: value.pattern.into(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpectrogramParameters {
    pub pattern: Pattern,
    pub gain: f32,
}

impl Default for SpectrogramParameters {
    fn default() -> Self {
        Self {
            pattern: Default::default(),
            gain: 1.5,
        }
    }
}

impl From<SpectrogramParameters> for shaders::SpectrogramParameters {
    fn from(value: SpectrogramParameters) -> Self {
        Self {
            pattern: Padded(value.pattern.into()),
            gain: value.gain,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrayVenueGridnodeParameters {
    pub lightness: f32,
    pub chroma: f32,
}

impl Default for GrayVenueGridnodeParameters {
    fn default() -> Self {
        Self {
            lightness: 0.8,
            chroma: 0.1,
        }
    }
}

impl From<GrayVenueGridnodeParameters> for shaders::GrayVenueGridnodeParameters {
    fn from(value: GrayVenueGridnodeParameters) -> Self {
        Self {
            lightness: value.lightness.into(),
            chroma: value.chroma,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImageParameters {
    pub scale_min: f32,
    pub scale_max: f32,
}

impl Default for ImageParameters {
    fn default() -> Self {
        Self {
            scale_min: 0.5,
            scale_max: 1.0,
        }
    }
}

impl From<ImageParameters> for shaders::ImageParameters {
    fn from(value: ImageParameters) -> Self {
        Self {
            scale_min: value.scale_min.into(),
            scale_max: value.scale_max,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MaskedPatternParameters {
    pub pattern: Pattern,
    pub scale_min: f32,
    pub scale_max: f32,
}

impl Default for MaskedPatternParameters {
    fn default() -> Self {
        Self {
            pattern: Default::default(),
            scale_min: 0.5,
            scale_max: 1.0,
        }
    }
}

impl From<MaskedPatternParameters> for shaders::MaskedPatternParameters {
    fn from(value: MaskedPatternParameters) -> Self {
        Self {
            pattern: Padded(value.pattern.into()),
            scale_min: value.scale_min.into(),
            scale_max: value.scale_max,
        }
    }
}
