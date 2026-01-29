use super::video::{
    Panel,
    PanelMaterial::{GrayVenueGridnode, Image, Rainbow, Spectrogram, Waveform},
    PanelPosition, PanelScale, PanelTransform,
    shaders::{
        GrayVenueGridnodeParameters, ImageParameters, RainbowParameters, SpectrogramParameters,
        WaveformParameters,
    },
};
use glam::{Vec2, vec2};
use std::f32::consts::FRAC_PI_2;
use winit::dpi::LogicalSize;

pub const WINDOW_SIZE: LogicalSize<i32> = LogicalSize::new(1920, 1080);
const WINDOW_SIZE_VEC: Vec2 = vec2(WINDOW_SIZE.width as f32, WINDOW_SIZE.height as f32);

pub const SAMPLE_COUNT: usize = 8192;
pub const BIN_COUNT: usize = 256;
pub const SAMPLE_RATE: u32 = 48000;

pub const PANELS: &[Panel] = &[
    // Panel {
    //     variant: WAVEFORM(WaveformParameters::DEFAULT),
    //     transform: PanelTransform::DEFAULT,
    // },
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
        material: Rainbow(RainbowParameters::DEFAULT),
        transform: PanelTransform::from_upper_left_corner_pixels(
            vec2(210.0, 800.0),
            vec2(1710.0, 0.0),
        )
        .rotate_ccw(),
    },
    Panel {
        material: Spectrogram(SpectrogramParameters::DEFAULT),
        transform: PanelTransform::from_upper_left_corner_pixels(
            vec2(210.0, 400.0),
            vec2(1710.0, 0.0),
        )
        .flip_y()
        .rotate_ccw(),
    },
    Panel {
        material: Spectrogram(SpectrogramParameters::DEFAULT),
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
];
