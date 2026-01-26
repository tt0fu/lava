use super::{
    GlobalWrites, shaders,
    shaders::{
        AspectRatio, GrayVenueGridnodeParameters, RainbowParameters, SpectrogramParameters, Transform, WaveformParameters,
    },
};

use glam::{Mat3, Vec2, vec2};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferContents, allocator::SubbufferAllocator},
    descriptor_set::WriteDescriptorSet,
    device::Device,
    shader::EntryPoint,
};

#[derive(Clone, Copy)]
pub enum PanelVariant {
    Waveform(WaveformParameters),
    Spectrogram(SpectrogramParameters),
    Rainbow(RainbowParameters),
    GrayVenueGridnode(GrayVenueGridnodeParameters),
}

#[derive(Clone, Copy)]
pub struct PanelTransform {
    pub scale: Vec2,
    pub angle: f32,
    pub translation: Vec2,
}

const fn div(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x / b.x, a.y / b.y)
}

const fn add(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x + b.x, a.y + b.y)
}

const fn sub(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x - b.x, a.y - b.y)
}

impl PanelTransform {
    pub const DEFAULT: Self = Self {
        scale: vec2(1.0, 1.0),
        angle: 0.0,
        translation: vec2(0.0, 0.0),
    };

    pub const fn from_upper_left_corner(
        scale_pixels: Vec2,
        upper_left_corner: Vec2,
        screen_pixels: Vec2,
        angle: f32,
    ) -> Self {
        let scale = div(scale_pixels, screen_pixels);
        let screen_center = div(screen_pixels, vec2(2.0, 2.0));
        let panel_center = div(scale_pixels, vec2(2.0, 2.0));
        let translation = div(
            sub(add(upper_left_corner, panel_center), screen_center),
            screen_center,
        );

        Self {
            scale,
            angle,
            translation,
        }
    }

    pub const fn mirror_x(&self) -> Self {
        Self {
            scale: vec2(-self.scale.x, self.scale.y),
            ..*self
        }
    }

    pub fn get_matrix(&self, screen_scale: Vec2) -> Mat3 {
        let to_screen = Mat3::from_scale(1.0 / screen_scale);
        let to_normalized = Mat3::from_scale(screen_scale);

        let scale = Mat3::from_scale(self.scale);
        let angle = to_screen * Mat3::from_angle(self.angle) * to_normalized;
        let translation = Mat3::from_translation(self.translation);

        translation * angle * scale
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.scale.x.abs() / self.scale.y.abs()
    }
}

impl Default for PanelTransform {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy)]
pub struct Panel {
    pub variant: PanelVariant,
    pub transform: PanelTransform,
}

impl Panel {
    pub fn get_shader_entry_point(&self, device: &Arc<Device>) -> EntryPoint {
        let device_clone = device.clone();
        match self.variant {
            PanelVariant::Waveform(_) => shaders::load_waveform(device_clone),
            PanelVariant::Spectrogram(_) => shaders::load_spectrogram(device_clone),
            PanelVariant::Rainbow(_) => shaders::load_rainbow(device_clone),
            PanelVariant::GrayVenueGridnode(_) => shaders::load_gray_venue_gridnode(device_clone),
        }
        .unwrap()
        .entry_point("main")
        .unwrap()
    }

    fn create_write_descriptor_set<T: BufferContents>(
        uniform_buffer_allocator: &SubbufferAllocator,
        binding: u32,
        content: T,
    ) -> WriteDescriptorSet {
        let buffer = uniform_buffer_allocator.allocate_sized().unwrap();
        *buffer.write().unwrap() = content;
        WriteDescriptorSet::buffer(binding, buffer)
    }

    pub fn get_write_descriptor_sets(
        &self,
        uniform_buffer_allocator: &SubbufferAllocator,
        screen_scale: Vec2,
        global_writes: GlobalWrites,
    ) -> Vec<WriteDescriptorSet> {
        let transform_write = {
            let transform = self.transform.get_matrix(screen_scale);
            Self::create_write_descriptor_set(
                &uniform_buffer_allocator,
                0,
                Transform {
                    transform: [
                        transform.x_axis.to_array().into(),
                        transform.y_axis.to_array().into(),
                        transform.z_axis.to_array().into(),
                    ],
                },
            )
        };

        let aspect_ratio_write = Self::create_write_descriptor_set(
            &uniform_buffer_allocator,
            1,
            AspectRatio {
                aspect_ratio: ((screen_scale.x / screen_scale.y)
                    * self.transform.get_aspect_ratio())
                .into(),
            },
        );

        match self.variant {
            PanelVariant::Waveform(parameters) => vec![
                transform_write,
                aspect_ratio_write,
                global_writes.samples,
                global_writes.stabilization,
                global_writes.bass,
                Self::create_write_descriptor_set(&uniform_buffer_allocator, 10, parameters),
            ],
            PanelVariant::Spectrogram(parameters) => vec![
                transform_write,
                global_writes.dft,
                Self::create_write_descriptor_set(&uniform_buffer_allocator, 10, parameters),
            ],
            PanelVariant::Rainbow(parameters) => vec![
                transform_write,
                aspect_ratio_write,
                global_writes.bass,
                Self::create_write_descriptor_set(&uniform_buffer_allocator, 10, parameters),
            ],
            PanelVariant::GrayVenueGridnode(parameters) => vec![
                transform_write,
                global_writes.dft,
                global_writes.bass,
                Self::create_write_descriptor_set(&uniform_buffer_allocator, 10, parameters),
            ],
        }
    }
}
