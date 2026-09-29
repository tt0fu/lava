use std::f32::consts::PI;

use serde::Deserialize;

use crate::video::{
    math::{to_ndc, unit_to_local},
    shaders,
};
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

/// The reference against which a [`Scalar`] is measured.
#[derive(Clone, Copy, Deserialize)]
pub enum Unit {
    /// Pixels.
    #[serde(rename = "px")]
    Pixels,
    /// Fractions of the parent's width.
    #[serde(rename = "w")]
    ScreenWidth,
    /// Fractions of the parent's height.
    #[serde(rename = "h")]
    ScreenHeight,
}

/// A single component of a [`Vector`], measured in some [`Unit`].
#[derive(Clone, Copy)]
pub struct Scalar {
    pub value: f32,
    pub unit: Unit,
}

/// A 2d value whose components can each be measured in a different [`Unit`].
#[derive(Clone, Copy)]
pub struct Vector {
    pub x: Scalar,
    pub y: Scalar,
}

impl Scalar {
    /// Interprets this scalar as a distance and returns it in pixels. `parent_size` is the size of
    /// the parent rectangle.
    pub fn to_pixels(&self, parent_size: Vec2) -> f32 {
        match self.unit {
            Unit::Pixels => self.value,
            Unit::ScreenWidth => self.value * parent_size.x,
            Unit::ScreenHeight => self.value * parent_size.y,
        }
    }
}

impl Vector {
    /// Interprets this vector as a distance and returns it in pixels.
    pub fn to_pixels(&self, parent_size: Vec2) -> Vec2 {
        vec2(self.x.to_pixels(parent_size), self.y.to_pixels(parent_size))
    }
}

#[derive(Clone, Copy)]
pub struct Transform {
    /// The point of the panel that is placed at [`Self::anchor_position`], in `[0, 1]` panel
    /// space (see the [`anchor`] module).
    pub anchor_point: Vec2,
    /// Position of the anchor point relative to the top left corner of the parent
    pub anchor_position: Vector,
    pub scale: Vector,
    pub rotation: f32,
}

impl Transform {
    pub const FULLSCREEN: Self = Self {
        anchor_point: anchor::CENTER,
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

    /// The panel's size in pixels, resolved against the parent's size.
    pub fn size(&self, parent_size: Vec2) -> Vec2 {
        self.scale.to_pixels(parent_size)
    }

    /// The anchor's position in the parent's pixel space.
    pub fn anchor(&self, parent_size: Vec2) -> Vec2 {
        self.anchor_position.to_pixels(parent_size)
    }

    /// The panel's aspect ratio (width / height), which is well-defined because the transform
    /// preserves rectangles.
    pub fn aspect_ratio(&self, parent_size: Vec2) -> f32 {
        let size = self.size(parent_size);
        size.x.abs() / size.y.abs()
    }

    /// Matrix mapping the unit quad `[-1, 1]^2` to the parent's pixel space. The transform first
    /// scales, then rotates around the anchor, then moves the anchor to its position.
    pub fn matrix_px(&self, parent_size: Vec2) -> Mat3 {
        let size = self.size(parent_size);
        let anchor = self.anchor(parent_size);
        let anchor_local = self.anchor_point * size;

        Mat3::from_translation(anchor)
            * Mat3::from_angle(self.rotation / 180.0 * PI)
            * Mat3::from_translation(-anchor_local)
            * unit_to_local(size)
    }

    /// Matrix mapping the unit quad `[-1, 1]^2` to the normalized device coordinates of a target
    /// whose origin and size (in pixels) are given. `parent_size` is the size of the parent
    /// rectangle that the transform is expressed in.
    pub fn matrix_ndc(&self, parent_size: Vec2, target_origin: Vec2, target_size: Vec2) -> Mat3 {
        to_ndc(target_origin, target_size) * self.matrix_px(parent_size)
    }
}

/// Packs a 3x3 matrix and aspect ratio into the shader's `Transform` buffer layout.
pub fn transform_buffer(mat: Mat3, aspect_ratio: f32) -> shaders::Transform {
    shaders::Transform {
        mat: [
            mat.x_axis.to_array().into(),
            mat.y_axis.to_array().into(),
            mat.z_axis.to_array().into(),
        ],
        aspect_ratio,
    }
}
