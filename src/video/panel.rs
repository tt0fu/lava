use super::{
    super::config::{BIN_COUNT, SAMPLE_COUNT, SAMPLE_RATE},
    GlobalWrites, PanelTransform, shaders,
    shaders::{
        AspectRatio, GrayVenueGridnodeParameters, ImageParameters, RainbowParameters,
        SpectrogramParameters, Transform, WaveformParameters,
    },
};

use glam::Vec2;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferContents, allocator::SubbufferAllocator},
    descriptor_set::WriteDescriptorSet,
    device::Device,
    shader::EntryPoint,
};

#[derive(Clone, Copy)]
pub enum PanelMaterial {
    Waveform(WaveformParameters),
    Spectrogram(SpectrogramParameters),
    Rainbow(RainbowParameters),
    Image(ImageParameters),
    GrayVenueGridnode(GrayVenueGridnodeParameters),
}

#[derive(Clone, Copy)]
pub struct Panel {
    pub material: PanelMaterial,
    pub transform: PanelTransform,
}

impl Panel {
    pub fn get_shader_entry_point(&self, device: &Arc<Device>) -> EntryPoint {
        let device_clone = device.clone();
        match self.material {
            PanelMaterial::Waveform(_) => shaders::load_waveform(device_clone),
            PanelMaterial::Spectrogram(_) => shaders::load_spectrogram(device_clone),
            PanelMaterial::Rainbow(_) => shaders::load_rainbow(device_clone),
            PanelMaterial::Image(_) => shaders::load_image(device_clone),
            PanelMaterial::GrayVenueGridnode(_) => shaders::load_gray_venue_gridnode(device_clone),
        }
        .unwrap()
        .specialize(
            [
                (0, (SAMPLE_COUNT as u32).into()),
                (1, (BIN_COUNT as u32).into()),
                (2, (SAMPLE_RATE as u32).into()),
            ]
            .into_iter()
            .collect(),
        )
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
        screen_size: Vec2,
        global_writes: GlobalWrites,
    ) -> Vec<WriteDescriptorSet> {
        let transform_write = {
            let transform = self.transform.get_matrix(screen_size);
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
                aspect_ratio: ((screen_size.x / screen_size.y) * self.transform.get_aspect_ratio())
                    .into(),
            },
        );

        match self.material {
            PanelMaterial::Waveform(parameters) => vec![
                transform_write,
                aspect_ratio_write,
                global_writes.samples,
                global_writes.stabilization,
                global_writes.bass,
                Self::create_write_descriptor_set(&uniform_buffer_allocator, 10, parameters),
            ],
            PanelMaterial::Spectrogram(parameters) => vec![
                transform_write,
                global_writes.dft,
                Self::create_write_descriptor_set(&uniform_buffer_allocator, 10, parameters),
            ],
            PanelMaterial::Rainbow(parameters) => vec![
                transform_write,
                aspect_ratio_write,
                global_writes.bass,
                Self::create_write_descriptor_set(&uniform_buffer_allocator, 10, parameters),
            ],
            PanelMaterial::Image(parameters) => vec![
                transform_write,
                global_writes.bass,
                global_writes.image_sampler,
                global_writes.image_view,
                Self::create_write_descriptor_set(&uniform_buffer_allocator, 10, parameters),
            ],
            PanelMaterial::GrayVenueGridnode(parameters) => vec![
                transform_write,
                global_writes.dft,
                global_writes.bass,
                Self::create_write_descriptor_set(&uniform_buffer_allocator, 10, parameters),
            ],
        }
    }
}
