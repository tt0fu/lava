use super::shaders::{spectrogram, waveform};

use glam::Vec2;
use std::sync::Arc;
use vulkano::{device::Device, shader::EntryPoint};

#[derive(Clone, Copy)]
pub enum PanelVariant {
    WAVEFORM,
    SPECTROGRAM,
}

#[derive(Clone, Copy)]
pub struct Panel {
    pub variant: PanelVariant,
    pub scale: Vec2,
    pub angle: f32,
    pub translation: Vec2,
}

impl Panel {
    pub fn get_shader_entry_point(&self, device: &Arc<Device>) -> EntryPoint {
        match self.variant {
            PanelVariant::WAVEFORM => waveform::load(device.clone()),
            PanelVariant::SPECTROGRAM => spectrogram::load(device.clone()),
        }
        .unwrap()
        .entry_point("main")
        .unwrap()
    }
}
