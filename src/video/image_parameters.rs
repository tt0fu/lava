use std::sync::OnceLock;

use anyhow::{Result, anyhow};
use glam::Vec4;
use serde::Deserialize;
use vulkano_taskgraph::descriptor_set::SampledImageId;

use crate::video::{
    parameters::{ImageIds, TypedParameters},
    shaders,
};

#[derive(Deserialize)]
pub struct ImageParameters {
    pub image: String,
    pub multiply: Vec4,
    pub add: Vec4,
    pub band_weights: Vec4,
    pub min_size: f32,

    #[serde(skip)]
    resolved: OnceLock<SampledImageId>,
}

impl TypedParameters for ImageParameters {
    type Content = shaders::ImageParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            image: (*self
                .resolved
                .get()
                .expect("image parameter was not resolved before being written"))
            .into(),
            multiply: self.multiply.into(),
            add: self.add.into(),
            band_weights: self.band_weights.into(),
            min_size: self.min_size,
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
