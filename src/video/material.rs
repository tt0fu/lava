use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow, bail};
use vulkano::{device::Device, shader::EntryPoint};

use crate::video::{
    image_parameters::ImageParameters,
    material_parameters::{
        BandsParameters, ClockParameters, ColorParameters, GridnodeParameters, HnodeParameters,
        PatternParameters, SpectrogramParameters, WaveformParameters,
    },
    parameters::Parameters,
    shaders,
};

/// Maps every image name declared in the config to its index in `SceneData::images`.
pub type ImageNames<'a> = HashMap<&'a str, usize>;

/// A shader program a material can use.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Shader {
    Color,
    Clock,
    Waveform,
    Spectrogram,
    Bands,
    Image,
    Pattern,
    Gridnode,
    Hnode,
}

impl Shader {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "color" => Self::Color,
            "clock" => Self::Clock,
            "waveform" => Self::Waveform,
            "spectrogram" => Self::Spectrogram,
            "bands" => Self::Bands,
            "image" => Self::Image,
            "pattern" => Self::Pattern,
            "gridnode" => Self::Gridnode,
            "hnode" => Self::Hnode,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Clock => "clock",
            Self::Waveform => "waveform",
            Self::Spectrogram => "spectrogram",
            Self::Bands => "bands",
            Self::Image => "image",
            Self::Pattern => "pattern",
            Self::Gridnode => "gridnode",
            Self::Hnode => "hnode",
        }
    }

    fn load(self, device: &Arc<Device>) -> Option<EntryPoint> {
        let module = unsafe {
            match self {
                Self::Color => shaders::load_color(device),
                Self::Clock => shaders::load_clock(device),
                Self::Waveform => shaders::load_waveform(device),
                Self::Spectrogram => shaders::load_spectrogram(device),
                Self::Bands => shaders::load_bands(device),
                Self::Image => shaders::load_image(device),
                Self::Pattern => shaders::load_pattern(device),
                Self::Gridnode => shaders::load_gridnode(device),
                Self::Hnode => shaders::load_hnode(device),
            }
        }
        .ok()?;
        module.entry_point("main")
    }

    fn parse_parameters(
        self,
        value: serde_json::Value,
        image_names: &ImageNames<'_>,
    ) -> Result<Box<dyn Parameters>> {
        Ok(match self {
            Self::Color => Box::new(serde_json::from_value::<ColorParameters>(value)?),
            Self::Clock => Box::new(serde_json::from_value::<ClockParameters>(value)?),
            Self::Waveform => Box::new(serde_json::from_value::<WaveformParameters>(value)?),
            Self::Spectrogram => Box::new(serde_json::from_value::<SpectrogramParameters>(value)?),
            Self::Bands => Box::new(serde_json::from_value::<BandsParameters>(value)?),
            Self::Image => {
                let parameters = serde_json::from_value::<ImageParameters>(value)?;
                if !image_names.contains_key(parameters.image.as_str()) {
                    bail!("material references unknown image '{}'", parameters.image);
                }
                Box::new(parameters)
            }
            Self::Pattern => Box::new(serde_json::from_value::<PatternParameters>(value)?),
            Self::Gridnode => Box::new(serde_json::from_value::<GridnodeParameters>(value)?),
            Self::Hnode => Box::new(serde_json::from_value::<HnodeParameters>(value)?),
        })
    }
}

/// A material and its shader parameters, ready to be written to a GPU buffer.
pub struct Material {
    /// Index into `SceneData::shaders`.
    pub shader_id: usize,
    pub parameters: Box<dyn Parameters>,
}

/// Loads each distinct shader program exactly once while building the scene.
#[derive(Default)]
pub struct ShaderCache {
    ids: HashMap<Shader, usize>,
    entry_points: Vec<EntryPoint>,
}

impl ShaderCache {
    fn id(&mut self, device: &Arc<Device>, shader: Shader) -> Result<usize> {
        if let Some(&id) = self.ids.get(&shader) {
            return Ok(id);
        }
        let entry_point = shader
            .load(device)
            .ok_or_else(|| anyhow!("failed to load shader '{}'", shader.name()))?;

        let id = self.entry_points.len();
        self.entry_points.push(entry_point);
        self.ids.insert(shader, id);
        Ok(id)
    }

    pub fn into_entry_points(self) -> Vec<EntryPoint> {
        self.entry_points
    }
}

pub fn create_material(
    device: &Arc<Device>,
    shader_name: &str,
    parameters: serde_json::Value,
    image_names: &ImageNames<'_>,
    shader_cache: &mut ShaderCache,
) -> Result<Material> {
    let shader =
        Shader::from_name(shader_name).ok_or_else(|| anyhow!("unknown shader '{shader_name}'"))?;
    Ok(Material {
        shader_id: shader_cache.id(device, shader)?,
        parameters: shader.parse_parameters(parameters, image_names)?,
    })
}
