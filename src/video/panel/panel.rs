use std::collections::HashMap;

use glam::{Vec2, Vec3, Vec4};
use vulkano::{buffer::Buffer, shader::EntryPoint};
use vulkano_taskgraph::Id;

use crate::video::panel::transform::Transform;

/*
{
  settings: {

  },
  custom_shaders: {
    "custom_shader": {
      source: "./custom_shader.glsl"
      parameters: [
        {
          name: "foo",
          type: "int"
        }
        {
          name: "bar",
          type: "int"
        }
      ]
    }
  },
  transforms: {
    "left_screen": {
      "left": { "pixels": [-215.0, 215.0] }, // top_left, left, bottom_left, top_right, right, bottom_right, top, bottom
      "scale": { "screen": [0.5, 0.5] },
      "rotation": 45.0,
      "order": 2
    }
  }
  materials: {
    "waveform_slim": {
      shader: "waveform"
      width: 0.5
    }
  }
  panels: [
     {
       transform: "left_screen", // or just the attrset for the transform
       material: "waveform_slim",
     }
  ]
}
*/

pub struct TransformPair {
    source: Transform,
    destination: Id<Buffer>,
}

pub enum Property {
    Bool(bool),
    Int(i32),
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}

pub struct Material {
    shader_id: usize,
    properties: HashMap<String, Property>,
}

pub struct RuntimeInfo {
    shaders: Vec<EntryPoint>,
    transforms: Vec<TransformPair>,
    materials: Vec<Material>,
}

pub struct Panel {
    transform_id: usize,
    material_id: usize,
}
