use std::f32::consts::PI;
use vulkano::buffer::Buffer;
use vulkano_taskgraph::{Id, TaskContext};

use crate::video::shaders;
use glam::{Mat3, Vec2, vec2};

pub mod anchor {
    use glam::{Vec2, vec2};

    pub const CENTER: Vec2 = vec2(0.5, 0.5);
    pub const TOP: Vec2 = vec2(0.5, 0.0);
    pub const BOTTOM: Vec2 = vec2(0.5, 1.0);
    pub const LEFT: Vec2 = vec2(0.0, 0.5);
    pub const RIGHT: Vec2 = vec2(1.0, 0.5);
    pub const TOP_LEFT: Vec2 = vec2(0.0, 0.0);
    pub const TOP_RIGHT: Vec2 = vec2(1.0, 0.0);
    pub const BOTTOM_LEFT: Vec2 = vec2(0.0, 1.0);
    pub const BOTTOM_RIGHT: Vec2 = vec2(1.0, 1.0);
}

pub enum Unit {
    Pixels,
    Screen,
}

pub struct Vector {
    pub value: Vec2,
    pub unit: Unit,
}

pub struct Transform {
    /// Position of the anchor point relative to the top left corner of the panel.
    /// See the anchor module
    pub anchor_type: Vec2,
    /// Position of the anchor point relative to the top left corner of the screen
    pub anchor_position: Vector,
    pub scale: Vector,
    pub rotation: f32,
}

impl Transform {
    pub const FULLSCREEN: Self = Self {
        anchor_type: anchor::CENTER,
        anchor_position: Vector {
            value: vec2(0.5, 0.5),
            unit: Unit::Screen,
        },
        scale: Vector {
            value: vec2(1.0, 1.0),
            unit: Unit::Screen,
        },
        rotation: 0.0,
    };

    /// Get the screen-relative scale of the panel. Full screen is (1, 1)
    pub fn get_scale(&self, resolution: Vec2) -> Vec2 {
        match self.scale.unit {
            Unit::Screen => self.scale.value,
            Unit::Pixels => self.scale.value / resolution,
        }
    }

    /// Get the normalized device coordinates of the center of the panel. Center of the screen is (0, 0), bottom right corner is (1, 1)
    pub fn get_translation(&self, resolution: Vec2) -> Vec2 {
        let anchor_pixels = match self.anchor_position.unit {
            Unit::Screen => self.anchor_position.value * resolution,
            Unit::Pixels => self.anchor_position.value,
        };
        let anchor_norm = 2.0 * anchor_pixels / resolution - Vec2::ONE;
        anchor_norm + (Vec2::ONE - self.anchor_type * 2.0)
    }

    /// Get a vertex shader ready 2d transformation matrix:
    /// gl_Position = vec4((mat * vec3(vertex_position, 1.0)).xy, 0.0, 1.0);
    pub fn get_matrix(&self, resolution: Vec2) -> Mat3 {
        let scale = Mat3::from_scale(self.get_scale(resolution));

        let translation = Mat3::from_translation(self.get_translation(resolution));

        let angle = Mat3::from_scale(1.0 / resolution)
            * Mat3::from_angle(self.rotation / 180.0 * PI)
            * Mat3::from_scale(resolution);

        translation * angle * scale
    }

    pub fn get_buffer(&self, resolution: Vec2) -> shaders::Transform {
        let mat = self.get_matrix(resolution);
        shaders::Transform {
            mat: [
                mat.x_axis.to_array().into(),
                mat.y_axis.to_array().into(),
                mat.z_axis.to_array().into(),
            ],
        }
    }

    pub fn write(&self, resolution: Vec2, id: Id<Buffer>, tcx: &mut TaskContext<'_>) {
        *tcx.write_buffer(id, ..) = self.get_buffer(resolution);
    }
}
