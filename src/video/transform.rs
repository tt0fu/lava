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
    /// Position of the anchor point relative to the top left corner of the panel.
    /// See the anchor module
    pub anchor_type: Vec2,
    /// Position of the anchor point relative to the top left corner of the parent
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

    /// Matrix mapping the unit quad `[-1, 1]²` to the parent's pixel space. The transform first
    /// scales, then rotates around the anchor, then moves the anchor to its position.
    pub fn matrix_px(&self, parent_size: Vec2) -> Mat3 {
        let size = self.size(parent_size);
        let anchor = self.anchor(parent_size);
        let anchor_local = self.anchor_type * size;

        Mat3::from_translation(anchor)
            * Mat3::from_angle(self.rotation / 180.0 * PI)
            * Mat3::from_translation(-anchor_local)
            * unit_to_local(size)
    }

    /// Matrix mapping the unit quad `[-1, 1]²` to the normalized device coordinates of a target
    /// whose origin and size (in pixels) are given. `parent_size` is the size of the parent
    /// rectangle that the transform is expressed in.
    pub fn matrix_ndc(&self, parent_size: Vec2, target_origin: Vec2, target_size: Vec2) -> Mat3 {
        to_ndc(target_origin, target_size) * self.matrix_px(parent_size)
    }

    /// Get a vertex shader ready 2d transformation matrix:
    /// gl_Position = vec4((mat * vec3(vertex_position, 1.0)).xy, 0.0, 1.0);
    pub fn get_matrix(&self, resolution: Vec2) -> Mat3 {
        self.matrix_ndc(resolution, Vec2::ZERO, resolution)
    }

    pub fn get_buffer(&self, resolution: Vec2) -> shaders::Transform {
        transform_buffer(self.get_matrix(resolution), self.aspect_ratio(resolution))
    }

    pub fn write(&self, resolution: Vec2, id: Id<Buffer>, tcx: &mut TaskContext<'_>) {
        *tcx.write_buffer(id, ..) = self.get_buffer(resolution);
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

/// Matrix mapping the unit quad `[-1, 1]²` to a rectangle's local `[0, size]` space.
pub fn unit_to_local(size: Vec2) -> Mat3 {
    Mat3::from_scale(size)
        * Mat3::from_translation(Vec2::splat(0.5))
        * Mat3::from_scale(Vec2::splat(0.5))
}

/// Matrix mapping pixel coordinates to the normalized device coordinates of a target with the
/// given origin and size (in pixels).
pub fn to_ndc(target_origin: Vec2, target_size: Vec2) -> Mat3 {
    Mat3::from_translation(-Vec2::ONE)
        * Mat3::from_scale(2.0 / target_size)
        * Mat3::from_translation(-target_origin)
}

/// The axis-aligned bounding box of `points`, snapped to integer pixel coordinates and fully
/// covering them. Returns `(origin, size)` with `size` at least `(1, 1)`.
pub fn aabb(points: &[Vec2]) -> (Vec2, Vec2) {
    let min = points
        .iter()
        .copied()
        .fold(Vec2::splat(f32::INFINITY), |a, b| a.min(b));
    let max = points
        .iter()
        .copied()
        .fold(Vec2::splat(f32::NEG_INFINITY), |a, b| a.max(b));
    let origin = min.floor();
    let size = (max.ceil() - origin).max(Vec2::ONE);
    (origin, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec3;

    fn ndc(
        t: &Transform,
        parent_size: Vec2,
        target_origin: Vec2,
        target_size: Vec2,
        q: Vec2,
    ) -> Vec2 {
        (t.matrix_ndc(parent_size, target_origin, target_size) * vec3(q.x, q.y, 1.0)).truncate()
    }

    fn scalar(value: f32, unit: Unit) -> Scalar {
        Scalar { value, unit }
    }

    fn transform(
        anchor_type: Vec2,
        position: [Scalar; 2],
        scale: [Scalar; 2],
        rotation: f32,
    ) -> Transform {
        Transform {
            anchor_type,
            anchor_position: Vector {
                x: position[0],
                y: position[1],
            },
            scale: Vector {
                x: scale[0],
                y: scale[1],
            },
            rotation,
        }
    }

    #[test]
    fn size_units() {
        let parent = vec2(1920.0, 1080.0);
        // w/h/px each resolve independently per axis.
        assert_eq!(
            Vector {
                x: scalar(0.5, Unit::ScreenWidth),
                y: scalar(0.5, Unit::ScreenHeight),
            }
            .to_pixels(parent),
            vec2(960.0, 540.0)
        );
        assert_eq!(
            Vector {
                x: scalar(0.5, Unit::ScreenHeight),
                y: scalar(100.0, Unit::Pixels),
            }
            .to_pixels(parent),
            vec2(540.0, 100.0)
        );
    }

    #[test]
    fn fullscreen_maps_quad_to_full_ndc() {
        let res = vec2(1920.0, 1080.0);
        let t = Transform::FULLSCREEN;
        // Center of the screen.
        assert!(ndc(&t, res, Vec2::ZERO, res, Vec2::ZERO).length() < 1e-6);
        // Top-left corner maps to NDC (-1, -1).
        let corner = ndc(&t, res, Vec2::ZERO, res, vec2(-1.0, -1.0));
        assert!((corner - vec2(-1.0, -1.0)).length() < 1e-5);
    }

    #[test]
    fn anchor_lands_on_position_under_rotation() {
        let res = vec2(1920.0, 1080.0);
        let t = transform(
            anchor::BOTTOM_LEFT,
            [
                scalar(0.25, Unit::ScreenWidth),
                scalar(0.75, Unit::ScreenHeight),
            ],
            [
                scalar(0.5, Unit::ScreenWidth),
                scalar(0.5, Unit::ScreenHeight),
            ],
            37.0,
        );
        let expected = vec2(2.0 * 0.25 - 1.0, 2.0 * 0.75 - 1.0);
        let local_anchor = 2.0 * t.anchor_type - Vec2::ONE;
        let got = ndc(&t, res, Vec2::ZERO, res, local_anchor);
        assert!(
            (got - expected).length() < 1e-5,
            "got {got:?} expected {expected:?}"
        );
    }

    #[test]
    fn matrix_ndc_offset_target() {
        // A transform rendered into an offset target must land in the same absolute pixels. The
        // fullscreen transform maps the unit quad to parent pixels linearly: p = (960 q.x + 960,
        // 540 q.y + 540).
        let parent = vec2(1920.0, 1080.0);
        let t = Transform::FULLSCREEN;
        let origin = vec2(0.0, 0.0);
        let target = vec2(960.0, 540.0);

        // Parent pixel (1440, 810) is outside the top-left quadrant target -> NDC (2, 2).
        let q = vec2(0.5, 0.5); // -> parent (1440, 810)
        let got = ndc(&t, parent, origin, target, q);
        assert!((got - vec2(2.0, 2.0)).length() < 1e-5, "got {got:?}");

        // Parent pixel (480, 270) is the target's own center -> NDC (0, 0).
        let q = vec2(-0.5, -0.5); // -> parent (480, 270)
        let got = ndc(&t, parent, origin, target, q);
        assert!(got.length() < 1e-5, "got {got:?}");
    }

    #[test]
    fn aabb_snaps_outward_to_integers() {
        let points = [vec2(10.3, 20.7), vec2(-4.2, 55.1), vec2(30.9, -0.5)];
        let (origin, size) = aabb(&points);
        assert_eq!(origin, vec2(-5.0, -1.0));
        assert_eq!(size, vec2(36.0, 57.0));
        // Fully covers all points.
        for p in points {
            assert!(p.x >= origin.x && p.y >= origin.y);
            assert!(p.x <= origin.x + size.x && p.y <= origin.y + size.y);
        }
    }
}
