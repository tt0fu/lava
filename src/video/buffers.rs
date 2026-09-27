use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
};
use vulkano_taskgraph::{
    Id,
    descriptor_set::{BindlessContext, StorageBufferId},
    resource::Resources,
};

use crate::{
    audio::{audio_settings::AudioSettings, stream::Stream},
    video::{
        global_parameters::GlobalParameters,
        parameters::Layout,
        scene_data::{Panel, SceneData},
        shaders::{self, ComputePushConstants, PushConstants},
    },
};

pub struct Buffers {
    pub global: Id<Buffer>,
    pub waveform: Id<Buffer>,
    pub dft: Id<Buffer>,
    pub bands: Id<Buffer>,

    pub transforms: Vec<Id<Buffer>>,
    pub materials: Vec<Id<Buffer>>,
}

impl Buffers {
    pub fn new(
        audio_settings: &AudioSettings,
        scene_data: &SceneData,
        resources: &Resources,
        stream: &Stream,
        global_parameters: &GlobalParameters,
    ) -> Self {
        let buffer_create_info = BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        };
        let allocation_create_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        };

        let create_buffer = |layout| {
            resources
                .create_buffer(&buffer_create_info, &allocation_create_info, layout)
                .unwrap()
        };
        Self {
            global: create_buffer(global_parameters.layout()),
            waveform: create_buffer(stream.layout()),
            dft: create_buffer(
                DeviceLayout::new_unsized::<shaders::Dft>(audio_settings.dft_bin_count as u64)
                    .unwrap(),
            ),
            bands: create_buffer(
                DeviceLayout::new_unsized::<shaders::Bands>(
                    audio_settings.bands_history_length as u64,
                )
                .unwrap(),
            ),
            transforms: scene_data
                .transforms
                .iter()
                .map(|_| create_buffer(DeviceLayout::new_sized::<shaders::Transform>()))
                .collect(),
            materials: scene_data
                .materials
                .iter()
                .map(|m| create_buffer(m.parameters.layout()))
                .collect(),
        }
    }
}

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
                .collect::<Vec<StorageBufferId>>(),
            materials: buffers
                .materials
                .iter()
                .map(create_storage_buffer)
                .collect::<Vec<StorageBufferId>>(),
        }
    }

    pub fn push_constants(&self, panel: &Panel, min_order: u32, max_order: u32) -> PushConstants {
        shaders::PushConstants {
            global_buffer_id: self.global,
            waveform_buffer_id: self.waveform,
            dft_buffer_id: self.dft,
            bands_buffer_id: self.bands,

            transform_buffer_id: self.transforms[panel.transform_id],
            material_buffer_id: self.materials[panel.material_id],

            panel_depth: (max_order - panel.order) as f32 / (max_order + 1 - min_order) as f32,
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
