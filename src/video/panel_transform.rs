use glam::{Mat3, Vec2, vec2};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PanelScale {
    #[serde(rename = "screen")]
    Screen(Vec2),
    #[serde(rename = "pixels")]
    Pixels(Vec2),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PanelPosition {
    #[serde(rename = "screen")]
    Screen(Vec2),
    #[serde(rename = "pixels")]
    Pixels(Vec2),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelTransform {
    pub scale: PanelScale,
    pub position: PanelPosition,
    pub angle: f32,
}

const fn add(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x + b.x, a.y + b.y)
}

const fn sub(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x - b.x, a.y - b.y)
}

const fn div(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x / b.x, a.y / b.y)
}

const fn mul(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x * b.x, a.y * b.y)
}

impl PanelTransform {
    pub const FULLSCREEN: Self = Self {
        scale: PanelScale::Screen(vec2(1.0, 1.0)),
        position: PanelPosition::Screen(vec2(0.5, 0.5)),
        angle: 0.0,
    };

    pub const fn from_upper_left_corner_pixels(
        size: Vec2,              // the size of the panel in pixels
        upper_left_corner: Vec2, // the position of the upper-left corner of the panel in pixels relative to the upper-left corner of the screen
    ) -> Self {
        let panel_center = div(size, vec2(2.0, 2.0));
        let position = add(upper_left_corner, panel_center);
        Self {
            scale: PanelScale::Pixels(size),
            position: PanelPosition::Pixels(position),
            angle: 0.0,
        }
    }

    pub fn get_matrix(&self, screen_size: Vec2) -> Mat3 {
        let to_screen = Mat3::from_scale(1.0 / screen_size);
        let to_normalized = Mat3::from_scale(screen_size);

        let scale = Mat3::from_scale(match self.scale {
            PanelScale::Screen(scale) => scale,
            PanelScale::Pixels(scale) => scale / screen_size,
        });

        let translation = Mat3::from_translation(match self.position {
            PanelPosition::Screen(position) => position * 2.0 - vec2(1.0, 1.0),
            PanelPosition::Pixels(position) => (position * 2.0) / screen_size - vec2(1.0, 1.0),
        });

        let angle = to_screen * Mat3::from_angle(self.angle / 180.0 * PI) * to_normalized;

        translation * angle * scale
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        let scale = match self.scale {
            PanelScale::Screen(scale) => scale,
            PanelScale::Pixels(scale) => scale,
        };
        scale.x.abs() / scale.y.abs()
    }

    pub const fn flip_x(&self) -> Self {
        Self {
            scale: (match self.scale {
                PanelScale::Screen(scale) => PanelScale::Screen(mul(scale, vec2(-1.0, 1.0))),
                PanelScale::Pixels(scale) => PanelScale::Pixels(mul(scale, vec2(-1.0, 1.0))),
            }),
            ..*self
        }
    }

    pub const fn flip_y(&self) -> Self {
        Self {
            scale: (match self.scale {
                PanelScale::Screen(scale) => PanelScale::Screen(mul(scale, vec2(1.0, -1.0))),
                PanelScale::Pixels(scale) => PanelScale::Pixels(mul(scale, vec2(1.0, -1.0))),
            }),
            ..*self
        }
    }

    pub const fn rotate_cw(&self) -> Self {
        Self {
            scale: (match self.scale {
                PanelScale::Screen(scale) => PanelScale::Screen(vec2(scale.y, scale.x)),
                PanelScale::Pixels(scale) => PanelScale::Pixels(vec2(scale.y, scale.x)),
            }),
            angle: self.angle + 90.0,
            ..*self
        }
    }

    pub const fn rotate_ccw(&self) -> Self {
        Self {
            scale: (match self.scale {
                PanelScale::Screen(scale) => PanelScale::Screen(vec2(scale.y, scale.x)),
                PanelScale::Pixels(scale) => PanelScale::Pixels(vec2(scale.y, scale.x)),
            }),
            angle: self.angle - 90.0,
            ..*self
        }
    }
}

impl Default for PanelTransform {
    fn default() -> Self {
        Self::FULLSCREEN
    }
}
