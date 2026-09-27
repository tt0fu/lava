use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow, bail};
use glam::Vec3;
use vulkano::{device::Device, shader::EntryPoint};

use crate::{
    config::Config,
    video::{
        material_parameters::{
            BandsParameters, ClockParameters, SimpleParameters, SpectrogramParameters,
            WaveformParameters,
        },
        parameters::Parameters,
        shaders,
        transform::Transform,
    },
};

pub struct Material {
    pub shader_id: usize, // index in the shaders vector
    pub parameters: Box<dyn Parameters>,
}

pub struct Panel {
    pub transform_id: usize, // index in the transforms vector
    pub material_id: usize,  // index in the materials vector
    pub order: u32,
}

/// All immutable runtime data the renderer needs to render the scene
pub struct SceneData {
    pub shaders: Vec<EntryPoint>,
    pub transforms: Vec<Transform>,
    pub materials: Vec<Material>,
    pub panels: Vec<Panel>,
    pub background_color: Vec3,
}

impl SceneData {
    pub fn new(device: &Arc<Device>, config: &Config) -> Result<Self> {
        let mut transform_ids = HashMap::new();
        let mut transforms = Vec::new();
        for (name, transform) in &config.transforms {
            transform_ids.insert(name.as_str(), transforms.len());
            transforms.push(transform.to_transform()?);
        }

        let mut shader_ids: HashMap<&str, usize> = HashMap::new();
        let mut shaders = Vec::new();
        let mut material_ids = HashMap::new();
        let mut materials = Vec::new();
        for (name, material) in &config.materials {
            let shader_id = match shader_ids.get(material.shader.as_str()) {
                Some(&id) => id,
                None => {
                    let entry_point = load_shader(device, &material.shader)
                        .ok_or_else(|| anyhow!("unknown shader `{}`", material.shader))?;
                    let id = shaders.len();
                    shaders.push(entry_point);
                    shader_ids.insert(material.shader.as_str(), id);
                    id
                }
            };
            material_ids.insert(name.as_str(), materials.len());
            materials.push(Material {
                shader_id,
                parameters: material_parameters(&material.shader, material.parameters.clone())?,
            });
        }

        let mut panels = Vec::new();
        for panel in &config.panels {
            let transform_id = *transform_ids.get(panel.transform.as_str()).ok_or_else(|| {
                anyhow!("panel references unknown transform `{}`", panel.transform)
            })?;
            let material_id = *material_ids
                .get(panel.material.as_str())
                .ok_or_else(|| anyhow!("panel references unknown material `{}`", panel.material))?;
            panels.push(Panel {
                transform_id,
                material_id,
                order: panel.order,
            });
        }

        Ok(Self {
            shaders,
            transforms,
            materials,
            panels,
            background_color: config.background_color,
        })
    }
}

fn load_shader(device: &Arc<Device>, name: &str) -> Option<EntryPoint> {
    let module = unsafe {
        match name {
            "simple" => shaders::load_simple(device),
            "clock" => shaders::load_clock(device),
            "waveform" => shaders::load_waveform(device),
            "spectrogram" => shaders::load_spectrogram(device),
            "bands" => shaders::load_bands(device),
            _ => return None,
        }
    }
    .ok()?;
    module.entry_point("main")
}

fn material_parameters(shader: &str, parameters: serde_json::Value) -> Result<Box<dyn Parameters>> {
    Ok(match shader {
        "simple" => Box::new(serde_json::from_value::<SimpleParameters>(parameters)?),
        "clock" => Box::new(serde_json::from_value::<ClockParameters>(parameters)?),
        "waveform" => Box::new(serde_json::from_value::<WaveformParameters>(parameters)?),
        "spectrogram" => Box::new(serde_json::from_value::<SpectrogramParameters>(parameters)?),
        "bands" => Box::new(serde_json::from_value::<BandsParameters>(parameters)?),
        _ => bail!("unknown shader `{shader}`"),
    })
}
