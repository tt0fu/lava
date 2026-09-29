use vulkano_taskgraph::descriptor_set::{BindlessContext, SamplerId, StorageBufferId};

use crate::video::{
    buffers::Buffers,
    shaders::{self, ComputePushConstants, PushConstants},
};

/// The bindless descriptor handles for every [`Buffers`] entry.
pub struct StorageBuffers {
    pub global: StorageBufferId,
    pub waveform: StorageBufferId,
    pub dft: StorageBufferId,
    pub bands: StorageBufferId,

    pub transforms: Vec<StorageBufferId>,
    pub materials: Vec<StorageBufferId>,
}

impl StorageBuffers {
    pub fn new(bcx: &BindlessContext, buffers: &Buffers) -> Self {
        let global_set = bcx.global_set();
        let create_storage_buffer = |&id| global_set.create_storage_buffer(id, 0, None).unwrap();
        Self {
            global: create_storage_buffer(&buffers.global),
            waveform: create_storage_buffer(&buffers.waveform),
            dft: create_storage_buffer(&buffers.dft),
            bands: create_storage_buffer(&buffers.bands),
            transforms: buffers
                .transforms
                .iter()
                .map(create_storage_buffer)
                .collect(),
            materials: buffers
                .materials
                .iter()
                .map(create_storage_buffer)
                .collect(),
        }
    }

    pub fn push_constants(
        &self,
        material: usize,
        transform_index: usize,
        sampler_id: SamplerId,
    ) -> PushConstants {
        shaders::PushConstants {
            global_buffer_id: self.global,
            waveform_buffer_id: self.waveform,
            dft_buffer_id: self.dft,
            bands_buffer_id: self.bands,

            transform_buffer_id: self.transforms[transform_index],
            material_buffer_id: self.materials[material],

            sampler_id,
        }
    }

    pub fn compute_push_constants(&self) -> ComputePushConstants {
        ComputePushConstants {
            global_buffer_id: self.global,
            waveform_buffer_id: self.waveform,
            dft_buffer_id: self.dft,
            bands_buffer_id: self.bands,
        }
    }
}
