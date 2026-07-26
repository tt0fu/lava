use glam::Vec3;

use crate::video::{parameters::TypedParameters, shaders};

pub struct SimpleParameters {
    pub value: f32,
}

impl TypedParameters for SimpleParameters {
    type Content = shaders::SimpleParams;

    fn get_content(&self) -> Self::Content {
        Self::Content { value: self.value }
    }
}

pub struct ClockParameters {
    pub col: Vec3,
    pub speed: f32,
}

impl TypedParameters for ClockParameters {
    type Content = shaders::ClockParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            col: self.col.into(),
            speed: self.speed,
        }
    }
}

pub struct WaveformParameters {
    pub col: Vec3,
    pub line_width: f32,
    pub gain: f32,
}

impl TypedParameters for WaveformParameters {
    type Content = shaders::WaveformParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            col: self.col.into(),
            line_width: self.line_width,
            gain: self.gain,
        }
    }
}

pub struct SpectrogramParameters {
    pub col: Vec3,
    pub gain: f32,
}

impl TypedParameters for SpectrogramParameters {
    type Content = shaders::SpectrogramParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            col: self.col.into(),
            gain: self.gain,
        }
    }
}

pub struct BandsParameters {
    pub col: Vec3,
    pub gain: f32,
}

impl TypedParameters for BandsParameters {
    type Content = shaders::BandsParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            col: self.col.into(),
            gain: self.gain,
        }
    }
}
