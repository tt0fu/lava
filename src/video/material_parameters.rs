use std::sync::OnceLock;

use anyhow::{Result, anyhow};
use glam::{Vec3, Vec4};
use serde::Deserialize;
use vulkano_taskgraph::descriptor_set::SampledImageId;

use crate::video::{
    parameters::{ImageIds, TypedParameters},
    shaders,
};

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
    pub col: Vec3,
    pub gain: f32,
}

impl TypedParameters for SpectrogramParameters {
    type Content = shaders::SpectrogramParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            col: self.col.into(),
            gain: self.gain,
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

#[derive(Deserialize)]
pub struct ImageParameters {
    pub image: String,

    #[serde(skip)]
    resolved: OnceLock<SampledImageId>,
}

impl TypedParameters for ImageParameters {
    type Content = shaders::ImageParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            image: *self
                .resolved
                .get()
                .expect("image parameter was not resolved before being written"),
        }
    }

    fn resolve_images(&self, images: &ImageIds) -> Result<()> {
        let image = images.get(&self.image)?;
        self.resolved.set(image).map_err(|_| {
            anyhow!(
                "image parameter '{}' was resolved more than once",
                self.image
            )
        })?;
        Ok(())
    }
}
