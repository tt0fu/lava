use super::video::{
    Panel, PanelTransform,
    PanelVariant::{GrayVenueGridnode, Rainbow, Spectrogram, Waveform},
    shaders::{
        GrayVenueGridnodeParameters, RainbowParameters, SpectrogramParameters, WaveformParameters,
    },
};
use glam::{Vec2, vec2};
use std::f32::consts::FRAC_PI_2;
use winit::dpi::LogicalSize;

pub const WINDOW_SIZE: LogicalSize<i32> = LogicalSize::new(1920, 1080);
const WINDOW_SIZE_VEC: Vec2 = vec2(WINDOW_SIZE.width as f32, WINDOW_SIZE.height as f32);

pub const SAMPLE_COUNT: usize = 8192; // must be equal to the value in consts.glsl
pub const BIN_COUNT: usize = 256; // must be equal to the value in consts.glsl
pub const SAMPLE_RATE: u32 = 48000; // must be equal to the value in consts.glsl

pub const PANELS: &[Panel] = &[
    // Panel {
    //     variant: WAVEFORM(WaveformParameters::DEFAULT),
    //     transform: PanelTransform::DEFAULT,
    // },
    Panel {
        variant: Waveform(WaveformParameters::DEFAULT),
        transform: PanelTransform::from_upper_left_corner(
            vec2(1413.0, 400.0),
            vec2(0.0, 0.0),
            WINDOW_SIZE_VEC,
            0.0,
        )
        .mirror_x(),
    },
    Panel {
        variant: Waveform(WaveformParameters::DEFAULT),
        transform: PanelTransform::from_upper_left_corner(
            vec2(1413.0, 400.0),
            vec2(0.0, 400.0),
            WINDOW_SIZE_VEC,
            0.0,
        ),
    },
    Panel {
        variant: Rainbow(RainbowParameters::DEFAULT),
        transform: PanelTransform::from_upper_left_corner(
            vec2(800.0, 210.0),
            vec2(1415.0, 295.0),
            WINDOW_SIZE_VEC,
            -FRAC_PI_2,
        ),
    },
    Panel {
        variant: Spectrogram(SpectrogramParameters::DEFAULT),
        transform: PanelTransform::from_upper_left_corner(
            vec2(400.0, 210.0),
            vec2(1615.0, 495.0),
            WINDOW_SIZE_VEC,
            -FRAC_PI_2,
        ),
    },
    Panel {
        variant: Spectrogram(SpectrogramParameters::DEFAULT),
        transform: PanelTransform::from_upper_left_corner(
            vec2(400.0, 210.0),
            vec2(1615.0, 95.0),
            WINDOW_SIZE_VEC,
            -FRAC_PI_2,
        )
        .mirror_x(),
    },
    Panel {
        variant: GrayVenueGridnode(GrayVenueGridnodeParameters::DEFAULT),
        transform: PanelTransform::from_upper_left_corner(
            vec2(1920.0, 208.0),
            vec2(0.0, 872.0),
            WINDOW_SIZE_VEC,
            0.0,
        ),
    },
];
