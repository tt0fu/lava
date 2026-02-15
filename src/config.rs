use crate::video::{
    Panel,
    PanelMaterial::{
        GrayVenueGridnode, Image, MaskedPattern, SimplePattern, Spectrogram, Waveform,
    },
    PanelPosition, PanelScale, PanelTransform,
    shaders::{
        GrayVenueGridnodeParameters, ImageParameters, MaskedPatternParameters, Pattern,
        SimplePatternParameters, SpectrogramParameters, WaveformParameters,
    },
};
use glam::vec2;
use std::f32::consts::FRAC_PI_2;
use vulkano::padded::Padded;
use winit::dpi::LogicalSize;

#[derive(Clone)]
pub struct Config {
    pub channels: u16,
    pub fetch_buffer_size: u32,
    pub store_buffer_size: usize,

    pub sample_count: usize,
    pub bin_count: usize,
    pub sample_rate: u32,

    pub window_size: LogicalSize<i32>,
    pub panels: Vec<Panel>,

    pub time_frames: bool,
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
            window_size: LogicalSize::new(1920, 1080),
            panels: vec![Panel {
                material: Waveform(WaveformParameters {
                    gain: 0.75,
                    ..WaveformParameters::DEFAULT
                }),
                transform: PanelTransform::FULLSCREEN,
            }],
            time_frames: true,
        }
    }
}

impl Config {
    pub fn double_waveform() -> Self {
        Self {
            panels: vec![
                Panel {
                    material: Waveform(WaveformParameters {
                        gain: 0.7,
                        ..WaveformParameters::DEFAULT
                    }),
                    transform: PanelTransform {
                        scale: PanelScale::Screen(vec2(1.0, -0.5)),
                        position: PanelPosition::Screen(vec2(0.5, 0.2)),
                        angle: 0.0,
                    },
                },
                Panel {
                    material: Waveform(WaveformParameters {
                        gain: 0.7,
                        ..WaveformParameters::DEFAULT
                    }),
                    transform: PanelTransform {
                        scale: PanelScale::Screen(vec2(1.0, 0.5)),
                        position: PanelPosition::Screen(vec2(0.5, 0.8)),
                        angle: 0.0,
                    },
                },
                Panel {
                    material: MaskedPattern(MaskedPatternParameters::DEFAULT),
                    transform: PanelTransform {
                        scale: PanelScale::Pixels(vec2(500.0, 500.0)),
                        position: PanelPosition::Screen(vec2(0.5, 0.5)),
                        angle: 0.0,
                    },
                },
            ],
            ..Default::default()
        }
    }

    pub fn grey_venue() -> Self {
        Self {
            panels: vec![
                Panel {
                    material: Waveform(WaveformParameters::DEFAULT),
                    transform: PanelTransform::from_upper_left_corner_pixels(
                        vec2(1413.0, 400.0),
                        vec2(0.0, 0.0),
                    )
                    .flip_x(),
                },
                Panel {
                    material: Waveform(WaveformParameters::DEFAULT),
                    transform: PanelTransform::from_upper_left_corner_pixels(
                        vec2(1413.0, 400.0),
                        vec2(0.0, 400.0),
                    ),
                },
                Panel {
                    material: SimplePattern(SimplePatternParameters::DEFAULT),
                    transform: PanelTransform::from_upper_left_corner_pixels(
                        vec2(210.0, 800.0),
                        vec2(1710.0, 0.0),
                    )
                    .rotate_ccw(),
                },
                Panel {
                    material: Spectrogram(SpectrogramParameters {
                        pattern: Padded(Pattern {
                            use_rainbow: 0,
                            ..Pattern::DEFAULT
                        }),
                        ..SpectrogramParameters::DEFAULT
                    }),
                    transform: PanelTransform::from_upper_left_corner_pixels(
                        vec2(210.0, 400.0),
                        vec2(1710.0, 0.0),
                    )
                    .flip_y()
                    .rotate_ccw(),
                },
                Panel {
                    material: Spectrogram(SpectrogramParameters {
                        pattern: Padded(Pattern {
                            use_rainbow: 0,
                            ..Pattern::DEFAULT
                        }),
                        ..SpectrogramParameters::DEFAULT
                    }),
                    transform: PanelTransform::from_upper_left_corner_pixels(
                        vec2(210.0, 400.0),
                        vec2(1710.0, 400.0),
                    )
                    .rotate_ccw(),
                },
                Panel {
                    material: GrayVenueGridnode(GrayVenueGridnodeParameters::DEFAULT),
                    transform: PanelTransform::from_upper_left_corner_pixels(
                        vec2(1920.0, 208.0),
                        vec2(0.0, 872.0),
                    ),
                },
                Panel {
                    material: Image(ImageParameters::DEFAULT),
                    transform: PanelTransform {
                        scale: PanelScale::Pixels(vec2(250.0, 250.0)),
                        position: PanelPosition::Pixels(vec2(1710.0 + (210.0 / 2.0), 400.0)),
                        angle: -FRAC_PI_2,
                    },
                },
            ],
            ..Default::default()
        }
    }
}
