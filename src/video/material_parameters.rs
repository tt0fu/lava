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
pub struct SimpleParameters {
    pub value: f32,
}

impl TypedParameters for SimpleParameters {
    type Content = shaders::SimpleParams;

    fn get_content(&self) -> Self::Content {
        Self::Content { value: self.value }
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
    pub col: Vec3,
    pub line_width: f32,
    pub gain: f32,
}

impl TypedParameters for WaveformParameters {
    type Content = shaders::WaveformParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            col: self.col.into(),
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
