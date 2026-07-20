use glam::{Mat3, Vec2, vec2};
// use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::video::shaders;

// #[derive(Clone, Copy, Serialize, Deserialize)]
// #[serde(tag = "type", content = "value")]
pub enum AnchorType {
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
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
    pub anchor_type: AnchorType,
    /// Position of the anchor point relative to the top left corner of the screen
    pub anchor: Vector,
    pub scale: Vector,
    pub rotation: f32,
}

impl Transform {
    pub const FULLSCREEN: Self = Self {
        anchor_type: AnchorType::Center,
        anchor: Vector {
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

    /// Get the screen-relative position of the center of the panel. Center of the screen is (0, 0), bottom right corner is (1, 1)
    pub fn get_translation(&self, resolution: Vec2) -> Vec2 {
        let anchor_pixels = match self.anchor.unit {
            Unit::Screen => self.anchor.value * resolution,
            Unit::Pixels => self.anchor.value,
        };

        let anchor_norm = 2.0 * anchor_pixels / resolution - Vec2::ONE;

        let offset = self.get_scale(resolution)
            * (match self.anchor_type {
                AnchorType::Center => vec2(0.0, 0.0),
                AnchorType::Top => vec2(0.0, 1.0),
                AnchorType::Bottom => vec2(0.0, -1.0),
                AnchorType::Left => vec2(1.0, 0.0),
                AnchorType::Right => vec2(-1.0, 0.0),
                AnchorType::TopLeft => vec2(1.0, 1.0),
                AnchorType::TopRight => vec2(-1.0, 1.0),
                AnchorType::BottomLeft => vec2(1.0, -1.0),
                AnchorType::BottomRight => vec2(-1.0, -1.0),
            });
        anchor_norm + offset
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
}

impl Default for Transform {
    fn default() -> Self {
        Self::FULLSCREEN
    }
}
