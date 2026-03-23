use crate::video::{
    Panel, PanelMaterial::Waveform, PanelTransform, shader_types::WaveformParameters,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use winit::dpi::LogicalSize;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub channels: u16,
    pub fetch_buffer_size: u32,
    pub store_buffer_size: usize,

    pub sample_count: usize,
    pub bin_count: usize,
    pub sample_rate: u32,

    pub window_title: String,
    pub window_decorations: bool,
    pub window_size: LogicalSize<i32>,
    pub panels: Vec<Panel>,
    pub image_path: Option<PathBuf>,

    pub frame_times: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            channels: 1,
            fetch_buffer_size: 512,
            store_buffer_size: 2048,
            sample_count: 8192,
            bin_count: 256,
            sample_rate: 48000,
            window_title: "lava visualizer".to_string(),
            window_decorations: false,
            window_size: LogicalSize::new(1920, 1080),
            panels: vec![Panel {
                material: Waveform(WaveformParameters {
                    gain: 0.75,
                    ..Default::default()
                }),
                transform: PanelTransform::FULLSCREEN,
            }],
            image_path: None,
            frame_times: false,
        }
    }
}

impl Config {
    pub fn from_jsonc(path: &Path) -> Self {
        serde_json::from_value(json5::from_str(&fs::read_to_string(path).unwrap()).unwrap())
            .unwrap()
    }

    pub fn to_jsonc(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
