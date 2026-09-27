use std::f32::consts::PI;

use serde::Deserialize;
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

#[derive(Clone, Copy, Deserialize)]
pub enum Unit {
    #[serde(rename = "px")]
    Pixels,
    #[serde(rename = "w")]
    ScreenWidth,
    #[serde(rename = "h")]
    ScreenHeight,
}

#[derive(Clone, Copy)]
pub struct Scalar {
    pub value: f32,
    pub unit: Unit,
}

impl Scalar {
    /// Interprets this scalar as a distance and returns it in pixels.
    fn to_pixels(&self, resolution: Vec2) -> f32 {
        match self.unit {
            Unit::Pixels => self.value,
            Unit::ScreenWidth => self.value * resolution.x,
            Unit::ScreenHeight => self.value * resolution.y,
        }
    }

    /// Interprets this scalar as the size of the panel on an axis and returns it as a fraction of
    /// the screen on that axis. `axis_resolution` is the resolution of that axis.
    fn to_scale(&self, resolution: Vec2, axis_resolution: f32) -> f32 {
        let reference = match self.unit {
            Unit::Pixels => 1.0,
            Unit::ScreenWidth => resolution.x,
            Unit::ScreenHeight => resolution.y,
        };
        self.value * reference / axis_resolution
    }
}

#[derive(Clone, Copy)]
pub struct Vector {
    pub x: Scalar,
    pub y: Scalar,
}

impl Vector {
    /// Interprets this vector as a distance and returns it in pixels.
    fn to_pixels(&self, resolution: Vec2) -> Vec2 {
        vec2(self.x.to_pixels(resolution), self.y.to_pixels(resolution))
    }

    /// Interprets this vector as the size of the panel and returns it as a fraction of the screen
    /// on each axis. Full screen is `(1, 1)`.
    fn to_scale(&self, resolution: Vec2) -> Vec2 {
        vec2(
            self.x.to_scale(resolution, resolution.x),
            self.y.to_scale(resolution, resolution.y),
        )
    }
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
            x: Scalar {
                value: 0.5,
                unit: Unit::ScreenWidth,
            },
            y: Scalar {
                value: 0.5,
                unit: Unit::ScreenHeight,
            },
        },
        scale: Vector {
            x: Scalar {
                value: 1.0,
                unit: Unit::ScreenWidth,
            },
            y: Scalar {
                value: 1.0,
                unit: Unit::ScreenHeight,
            },
        },
        rotation: 0.0,
    };

    /// Get the screen-relative scale of the panel. Full screen is (1, 1)
    pub fn get_scale(&self, resolution: Vec2) -> Vec2 {
        self.scale.to_scale(resolution)
    }

    /// Get the normalized device coordinates of the anchor point.
    fn get_anchor(&self, resolution: Vec2) -> Vec2 {
        2.0 * self.anchor_position.to_pixels(resolution) / resolution - Vec2::ONE
    }

    /// Get the offset from the center of the panel to the anchor point, after scaling, in
    /// normalized device coordinates.
    fn get_anchor_offset(&self, resolution: Vec2) -> Vec2 {
        self.get_scale(resolution) * (2.0 * self.anchor_type - Vec2::ONE)
    }

    /// Get a vertex shader ready 2d transformation matrix:
    /// gl_Position = vec4((mat * vec3(vertex_position, 1.0)).xy, 0.0, 1.0);
    /// first scale, then rotate around the anchor, then move the anchor to its position.
    pub fn get_matrix(&self, resolution: Vec2) -> Mat3 {
        let scale = Mat3::from_scale(self.get_scale(resolution));

        let angle = Mat3::from_scale(1.0 / resolution)
            * Mat3::from_angle(self.rotation / 180.0 * PI)
            * Mat3::from_scale(resolution);

        Mat3::from_translation(self.get_anchor(resolution))
            * angle
            * Mat3::from_translation(-self.get_anchor_offset(resolution))
            * scale
    }

    pub fn get_buffer(&self, resolution: Vec2) -> shaders::Transform {
        let mat = self.get_matrix(resolution);
        let scale = self.get_scale(resolution) * resolution;
        shaders::Transform {
            mat: [
                mat.x_axis.to_array().into(),
                mat.y_axis.to_array().into(),
                mat.z_axis.to_array().into(),
            ],
            aspect_ratio: scale.x.abs() / scale.y.abs(),
        }
    }

    pub fn write(&self, resolution: Vec2, id: Id<Buffer>, tcx: &mut TaskContext<'_>) {
        *tcx.write_buffer(id, ..) = self.get_buffer(resolution);
    }
}
