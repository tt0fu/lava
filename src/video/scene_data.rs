use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result, anyhow};
use glam::Vec4;
use vulkano::{device::Device, shader::EntryPoint};

use crate::{
    config::{Config, ElementConfig, MaterialRef, TransformRef},
    video::{
        material::{ImageNames, Material, ShaderCache, create_material},
        scene_element::{Element, Group, Panel},
        transform::Transform,
    },
};

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
    pub background: Vec4,
    /// The root element (`panels`).
    pub root: Element,
}

impl SceneData {
    pub fn new(device: &Arc<Device>, config: &Config) -> Result<Self> {
        let (images, image_names) = load_images(config)?;
        let transforms = load_transforms(config)?;

        let mut builder = ElementBuilder {
            device,
            image_names: &image_names,
            transforms: &transforms,
            material_ids: HashMap::new(),
            shaders: ShaderCache::default(),
            materials: Vec::new(),
        };
        for (name, material) in &config.materials {
            builder
                .material_ids
                .insert(name.clone(), builder.materials.len());
            let created = create_material(
                builder.device,
                &material.shader,
                material.parameters.clone(),
                builder.image_names,
                &mut builder.shaders,
            )?;
            builder.materials.push(created);
        }
        let root = builder.build(&config.panels)?;

        Ok(Self {
            shaders: builder.shaders.into_entry_points(),
            images,
            materials: builder.materials,
            background: config.background,
            root,
        })
    }
}

fn load_images(config: &Config) -> Result<(Vec<SceneImage>, ImageNames<'_>)> {
    let mut image_names = HashMap::new();
    let mut images = Vec::new();
    for (name, path) in &config.images {
        let path = config.base_dir.join(path);
        let image = image::open(&path)
            .with_context(|| format!("failed to open image '{name}' at {}", path.display()))?
            .to_rgba8();
        let (width, height) = image.dimensions();
        image_names.insert(name.as_str(), images.len());
        images.push(SceneImage {
            name: name.clone(),
            width,
            height,
            data: image.into_raw(),
        });
    }
    Ok((images, image_names))
}

fn load_transforms(config: &Config) -> Result<HashMap<String, Transform>> {
    config
        .transforms
        .iter()
        .map(|(name, transform)| Ok((name.clone(), transform.to_transform()?)))
        .collect()
}

/// Recursively lowers the parsed config tree into [`Element`]s, resolving material and transform
/// references and registering inline materials as it goes.
struct ElementBuilder<'a> {
    device: &'a Arc<Device>,
    image_names: &'a ImageNames<'a>,
    transforms: &'a HashMap<String, Transform>,
    material_ids: HashMap<String, usize>,
    shaders: ShaderCache,
    materials: Vec<Material>,
}

impl ElementBuilder<'_> {
    fn build(&mut self, config: &ElementConfig) -> Result<Element> {
        match config {
            ElementConfig::Panel(panel) => Ok(Element::Panel(Panel {
                transform: self.resolve_transform(&panel.transform)?,
                material: self.resolve_material(&panel.material)?,
                order: panel.order,
                blend: panel.blend.to_blend()?,
            })),
            ElementConfig::Group(group) => {
                let children = group
                    .children
                    .iter()
                    .map(|child| self.build(child))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Element::Group(Group {
                    transform: self.resolve_transform(&group.transform)?,
                    background: group.background,
                    order: group.order,
                    blend: group.blend.to_blend()?,
                    children,
                }))
            }
        }
    }

    fn resolve_material(&mut self, config: &MaterialRef) -> Result<usize> {
        match config {
            MaterialRef::Named(name) => self
                .material_ids
                .get(name)
                .copied()
                .ok_or_else(|| anyhow!("panel references unknown material '{name}'")),
            MaterialRef::Inline(material) => {
                let created = create_material(
                    self.device,
                    &material.shader,
                    material.parameters.clone(),
                    self.image_names,
                    &mut self.shaders,
                )?;
                let index = self.materials.len();
                self.materials.push(created);
                Ok(index)
            }
        }
    }

    fn resolve_transform(&self, config: &TransformRef) -> Result<Transform> {
        match config {
            TransformRef::Named(name) => self
                .transforms
                .get(name)
                .copied()
                .ok_or_else(|| anyhow!("references unknown transform '{name}'")),
            TransformRef::Inline(config) => config.to_transform(),
        }
    }
}
