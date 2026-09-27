use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result, anyhow, bail};
use glam::Vec3;
use vulkano::{
    device::Device, pipeline::graphics::color_blend::AttachmentBlend, shader::EntryPoint,
};

use crate::{
    config::{Config, MaterialConfig, MaterialRef, TransformRef},
    video::{
        material_parameters::{
            BandsParameters, ClockParameters, ColorParameters, ImageParameters, PatternParameters,
            SpectrogramParameters, WaveformParameters,
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
    pub blend: AttachmentBlend,
}

pub struct SceneImage {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// All immutable runtime data the renderer needs to render the scene
pub struct SceneData {
    pub shaders: Vec<EntryPoint>,
    pub transforms: Vec<Transform>,
    pub images: Vec<SceneImage>,
    pub materials: Vec<Material>,
    pub panels: Vec<Panel>,
    pub background_color: Vec3,
}

impl SceneData {
    pub fn new(device: &Arc<Device>, config: &Config) -> Result<Self> {
        let mut image_ids = HashMap::new();
        let mut images = Vec::new();
        for (name, path) in &config.images {
            let path = config.base_dir.join(path);
            let image = image::open(&path)
                .with_context(|| format!("failed to open image '{name}' at {}", path.display()))?
                .to_rgba8();
            let (width, height) = image.dimensions();
            image_ids.insert(name.as_str(), images.len());
            images.push(SceneImage {
                name: name.clone(),
                width,
                height,
                data: image.into_raw(),
            });
        }

        let mut transform_ids = HashMap::new();
        let mut transforms = Vec::new();
        for (name, transform) in &config.transforms {
            transform_ids.insert(name.as_str(), transforms.len());
            transforms.push(transform.to_transform()?);
        }

        let mut shader_ids: HashMap<String, usize> = HashMap::new();
        let mut shaders = Vec::new();
        let mut material_ids = HashMap::new();
        let mut materials = Vec::new();
        for (name, material) in &config.materials {
            material_ids.insert(name.as_str(), materials.len());
            materials.push(create_material(
                device,
                material,
                &image_ids,
                &mut shader_ids,
                &mut shaders,
            )?);
        }

        let mut panels = Vec::new();
        for panel in &config.panels {
            let transform_id = match &panel.transform {
                TransformRef::Named(name) => *transform_ids
                    .get(name.as_str())
                    .ok_or_else(|| anyhow!("panel references unknown transform '{name}'"))?,
                TransformRef::Inline(config) => {
                    let id = transforms.len();
                    transforms.push(config.to_transform()?);
                    id
                }
            };
            let material_id = match &panel.material {
                MaterialRef::Named(name) => *material_ids
                    .get(name.as_str())
                    .ok_or_else(|| anyhow!("panel references unknown material '{name}'"))?,
                MaterialRef::Inline(config) => {
                    let id = materials.len();
                    materials.push(create_material(
                        device,
                        config,
                        &image_ids,
                        &mut shader_ids,
                        &mut shaders,
                    )?);
                    id
                }
            };
            panels.push(Panel {
                transform_id,
                material_id,
                order: panel.order,
                blend: panel.blend.to_blend()?,
            });
        }

        Ok(Self {
            shaders,
            transforms,
            images,
            materials,
            panels,
            background_color: config.background_color,
        })
    }
}

fn create_material(
    device: &Arc<Device>,
    config: &MaterialConfig,
    image_ids: &HashMap<&str, usize>,
    shader_ids: &mut HashMap<String, usize>,
    shaders: &mut Vec<EntryPoint>,
) -> Result<Material> {
    let shader_id = match shader_ids.get(&config.shader) {
        Some(&id) => id,
        None => {
            let entry_point = load_shader(device, &config.shader)
                .ok_or_else(|| anyhow!("unknown shader '{}'", config.shader))?;
            let id = shaders.len();
            shaders.push(entry_point);
            shader_ids.insert(config.shader.clone(), id);
            id
        }
    };
    let parameters = material_parameters(&config.shader, config.parameters.clone(), image_ids)?;
    Ok(Material {
        shader_id,
        parameters,
    })
}

fn load_shader(device: &Arc<Device>, name: &str) -> Option<EntryPoint> {
    let module = unsafe {
        match name {
            "color" => shaders::load_color(device),
            "clock" => shaders::load_clock(device),
            "waveform" => shaders::load_waveform(device),
            "spectrogram" => shaders::load_spectrogram(device),
            "bands" => shaders::load_bands(device),
            "image" => shaders::load_image(device),
            "pattern" => shaders::load_pattern(device),
            _ => return None,
        }
    }
    .ok()?;
    module.entry_point("main")
}

fn material_parameters(
    shader: &str,
    parameters: serde_json::Value,
    image_ids: &HashMap<&str, usize>,
) -> Result<Box<dyn Parameters>> {
    let parameters: Box<dyn Parameters> = match shader {
        "color" => Box::new(serde_json::from_value::<ColorParameters>(parameters)?),
        "clock" => Box::new(serde_json::from_value::<ClockParameters>(parameters)?),
        "waveform" => Box::new(serde_json::from_value::<WaveformParameters>(parameters)?),
        "spectrogram" => Box::new(serde_json::from_value::<SpectrogramParameters>(parameters)?),
        "bands" => Box::new(serde_json::from_value::<BandsParameters>(parameters)?),
        "image" => {
            let image_parameters = serde_json::from_value::<ImageParameters>(parameters)?;
            if !image_ids.contains_key(image_parameters.image.as_str()) {
                bail!(
                    "material references unknown image '{}'",
                    image_parameters.image
                );
            }
            Box::new(image_parameters)
        }
        "pattern" => Box::new(serde_json::from_value::<PatternParameters>(parameters)?),
        _ => bail!("unknown shader '{shader}'"),
    };
    Ok(parameters)
}
