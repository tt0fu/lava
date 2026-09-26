use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
};
use vulkano_taskgraph::{Id, resource::Resources};

use crate::{
    audio::stream::Stream,
    video::{
        audio_settings::AudioSettings, global_parameters::GlobalParameters, parameters::Layout,
        scene_data::SceneData, shaders,
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
