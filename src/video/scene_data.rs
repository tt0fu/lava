use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result, anyhow, bail};
use glam::Vec4;
use vulkano::{
    device::Device, pipeline::graphics::color_blend::AttachmentBlend, shader::EntryPoint,
};

use crate::{
    config::{Config, ElementConfig, MaterialConfig, MaterialRef, TransformRef},
    video::{
        material_parameters::{
            BandsParameters, ClockParameters, ColorParameters, GridnodeParameters, HnodeParameters,
            ImageParameters, PatternParameters, SpectrogramParameters, WaveformParameters,
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

pub enum Element {
    Panel(Panel),
    Group(Group),
}

pub struct Panel {
    pub transform: Transform,
    pub material: usize, // index in the materials vector
    pub order: u32,
    pub blend: AttachmentBlend,
}

pub struct Group {
    pub transform: Transform,
    pub background: Vec4,
    pub order: u32,
    pub blend: AttachmentBlend,
    pub children: Vec<Element>,
}

/// A decoded image, ready to be uploaded to the GPU as an RGBA8 texture.
pub struct SceneImage {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// All immutable runtime data the renderer needs to render the scene.
pub struct SceneData {
    pub shaders: Vec<EntryPoint>,
    pub images: Vec<SceneImage>,
    pub materials: Vec<Material>,
    pub background_color: Vec4,
    /// The root element (`panels`).
    pub root: Element,
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

        let mut transform_defs = HashMap::new();
        for (name, transform) in &config.transforms {
            transform_defs.insert(name.clone(), transform.to_transform()?);
        }

        let mut shader_ids: HashMap<String, usize> = HashMap::new();
        let mut shaders = Vec::new();
        let mut material_ids = HashMap::new();
        let mut materials = Vec::new();
        for (name, material) in &config.materials {
            material_ids.insert(name.clone(), materials.len());
            materials.push(create_material(
                device,
                material,
                &image_ids,
                &mut shader_ids,
                &mut shaders,
            )?);
        }

        let root = build_element(
            device,
            &image_ids,
            &transform_defs,
            &material_ids,
            &mut shader_ids,
            &mut shaders,
            &mut materials,
            &config.panels,
        )?;

        Ok(Self {
            shaders,
            images,
            materials,
            background_color: config.background_color,
            root,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn build_element(
    device: &Arc<Device>,
    image_ids: &HashMap<&str, usize>,
    transform_defs: &HashMap<String, Transform>,
    material_ids: &HashMap<String, usize>,
    shader_ids: &mut HashMap<String, usize>,
    shaders: &mut Vec<EntryPoint>,
    materials: &mut Vec<Material>,
    config: &ElementConfig,
) -> Result<Element> {
    match config {
        ElementConfig::Panel(panel) => {
            let material = match &panel.material {
                MaterialRef::Named(name) => *material_ids
                    .get(name)
                    .ok_or_else(|| anyhow!("panel references unknown material '{name}'"))?,
                MaterialRef::Inline(material) => {
                    let id = materials.len();
                    materials.push(create_material(
                        device, material, image_ids, shader_ids, shaders,
                    )?);
                    id
                }
            };
            Ok(Element::Panel(Panel {
                transform: resolve_transform(&panel.transform, transform_defs)?,
                material,
                order: panel.order,
                blend: panel.blend.to_blend()?,
            }))
        }
        ElementConfig::Group(group) => {
            let children = group
                .children
                .iter()
                .map(|child| {
                    build_element(
                        device,
                        image_ids,
                        transform_defs,
                        material_ids,
                        shader_ids,
                        shaders,
                        materials,
                        child,
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Element::Group(Group {
                transform: resolve_transform(&group.transform, transform_defs)?,
                background: group.background_color,
                order: group.order,
                blend: group.blend.to_blend()?,
                children,
            }))
        }
    }
}

fn resolve_transform(
    config: &TransformRef,
    transform_defs: &HashMap<String, Transform>,
) -> Result<Transform> {
    match config {
        TransformRef::Named(name) => transform_defs
            .get(name)
            .copied()
            .ok_or_else(|| anyhow!("references unknown transform '{name}'")),
        TransformRef::Inline(config) => config.to_transform(),
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
            "gridnode" => shaders::load_gridnode(device),
            "hnode" => shaders::load_hnode(device),
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
        "gridnode" => Box::new(serde_json::from_value::<GridnodeParameters>(parameters)?),
        "hnode" => Box::new(serde_json::from_value::<HnodeParameters>(parameters)?),
        _ => bail!("unknown shader '{shader}'"),
    };
    Ok(parameters)
}
