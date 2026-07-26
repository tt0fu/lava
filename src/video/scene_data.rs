use glam::Vec3;
use vulkano::shader::EntryPoint;

use crate::video::{parameters::Parameters, transform::Transform};

pub struct Material {
    pub shader_id: usize, // index in the shaders vector
    pub parameters: Box<dyn Parameters>,
}

pub struct Panel {
    pub transform_id: usize, // index in the transforms vector
    pub material_id: usize,  // index in the materials vector
    pub order: u32,
}

pub struct SceneData {
    pub shaders: Vec<EntryPoint>,
    pub transforms: Vec<Transform>,
    pub materials: Vec<Material>,
    pub panels: Vec<Panel>,
    pub background_color: Vec3,
}
