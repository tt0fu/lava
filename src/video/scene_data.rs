use glam::{Vec3, vec2, vec3, vec4};
use std::sync::Arc;
use vulkano::{device::Device, shader::EntryPoint};

use crate::video::material_parameters::{
    BandsParameters, SpectrogramParameters, WaveformParameters,
};
use crate::video::shaders::{self, specialize};
use crate::video::transform::{Unit, Vector, anchor};
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

/// All immutable runtime data the renderer needs to render the scene
pub struct SceneData {
    pub shaders: Vec<EntryPoint>,
    pub transforms: Vec<Transform>,
    pub materials: Vec<Material>,
    pub panels: Vec<Panel>,
    pub background_color: Vec3,
}

impl SceneData {
    pub fn new(device: &Arc<Device>) -> Self {
        Self {
            shaders: unsafe {
                vec![
                    shaders::load_simple(&device).unwrap(),
                    shaders::load_clock(&device).unwrap(),
                    shaders::load_waveform(&device).unwrap(),
                    shaders::load_spectrogram(&device).unwrap(),
                    shaders::load_bands(&device).unwrap(),
                ]
            }
            .iter()
            .map(|m| specialize(&m).entry_point("main").unwrap())
            .collect(),
            transforms: vec![
                Transform {
                    // bottom strip
                    anchor_type: anchor::BOTTOM_LEFT,
                    anchor_position: Vector {
                        value: vec2(0.0, 1.0),
                        unit: Unit::Screen,
                    },
                    scale: Vector {
                        value: vec2(1.0, 0.2),
                        unit: Unit::Screen,
                    },
                    rotation: 0.0,
                },
                Transform {
                    // middle strip
                    anchor_type: anchor::BOTTOM_LEFT,
                    anchor_position: Vector {
                        value: vec2(0.0, 0.8),
                        unit: Unit::Screen,
                    },
                    scale: Vector {
                        value: vec2(1.0, 0.4),
                        unit: Unit::Screen,
                    },
                    rotation: 0.0,
                },
                Transform {
                    // top strip
                    anchor_type: anchor::BOTTOM_LEFT,
                    anchor_position: Vector {
                        value: vec2(0.0, 0.4),
                        unit: Unit::Screen,
                    },
                    scale: Vector {
                        value: vec2(1.0, 0.4),
                        unit: Unit::Screen,
                    },
                    rotation: 0.0,
                },
            ],
            materials: vec![
                Material {
                    shader_id: 2,
                    parameters: Box::new(WaveformParameters {
                        col: vec3(1.0, 1.0, 1.0),
                        line_width: 50.0,
                        gain: 1.0,
                    }),
                },
                Material {
                    shader_id: 3,
                    parameters: Box::new(SpectrogramParameters {
                        col: vec3(1.0, 1.0, 1.0),
                        gain: 2.0,
                    }),
                },
                Material {
                    shader_id: 4,
                    parameters: Box::new(BandsParameters {
                        col: vec3(1.0, 1.0, 1.0),
                        gain: vec4(2.0, 2.0, 3.0, 8.0),
                    }),
                },
            ],
            panels: vec![
                Panel {
                    transform_id: 0,
                    material_id: 2,
                    order: 0,
                },
                Panel {
                    transform_id: 1,
                    material_id: 1,
                    order: 0,
                },
                Panel {
                    transform_id: 2,
                    material_id: 0,
                    order: 0,
                },
            ],
            background_color: vec3(0.0, 0.0, 0.0),
        }
    }
}
