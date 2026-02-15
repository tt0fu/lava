use crate::{
    audio::analyzer::AudioData,
    video::{
        Texture, create_write_descriptor_set,
        shaders::{Dft, Samples},
    },
};

use vulkano::{
    buffer::{Subbuffer, allocator::SubbufferAllocator},
    descriptor_set::WriteDescriptorSet,
};

#[derive(Clone)]
pub struct GlobalWrites {
    pub samples: WriteDescriptorSet,
    pub stabilization: WriteDescriptorSet,
    pub dft: WriteDescriptorSet,
    pub bass: WriteDescriptorSet,
    pub image_sampler: WriteDescriptorSet,
    pub image_view: WriteDescriptorSet,
}

impl GlobalWrites {
    pub fn new(
        uniform_buffer_allocator: &SubbufferAllocator,
        storage_buffer_allocator: &SubbufferAllocator,
        texture: &Texture,
        audio_data: &AudioData,
    ) -> Self {
        Self {
            samples: {
                let buffer: Subbuffer<Samples> = storage_buffer_allocator
                    .allocate_unsized(audio_data.samples.data.len() as u64)
                    .unwrap();
                let mut guard = buffer.write().unwrap();
                guard.samples_start = audio_data.samples.start as u32;
                guard.samples_data.copy_from_slice(&audio_data.samples.data);
                drop(guard);
                WriteDescriptorSet::buffer(2, buffer)
            },
            stabilization: create_write_descriptor_set(
                &uniform_buffer_allocator,
                3,
                audio_data.stabilization,
            ),
            dft: {
                let buffer: Subbuffer<Dft> = storage_buffer_allocator
                    .allocate_unsized(audio_data.dft.len() as u64)
                    .unwrap();
                let mut guard = buffer.write().unwrap();
                guard.dft.copy_from_slice(
                    &audio_data
                        .dft
                        .iter()
                        .map(|bin| [bin.x, bin.y].into())
                        .collect::<Vec<[f32; 2]>>(),
                );
                drop(guard);
                WriteDescriptorSet::buffer(4, buffer)
            },
            bass: create_write_descriptor_set(&uniform_buffer_allocator, 5, audio_data.bass),
            image_sampler: WriteDescriptorSet::sampler(6, texture.sampler.clone()),
            image_view: WriteDescriptorSet::image_view(7, texture.image_view.clone()),
        }
    }
}
