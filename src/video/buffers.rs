use glam::Vec4;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
};
use vulkano_taskgraph::{Id, TaskContext, resource::Resources};

use crate::{
    audio::{audio_settings::AudioSettings, stream::Stream},
    video::{
        global_parameters::GlobalParameters, parameters::Layout, scene_data::SceneData, shaders,
    },
};

pub struct Buffers {
    pub global: Id<Buffer>,
    pub waveform: Id<Buffer>,
    pub dft: Id<Buffer>,
    pub bands: Id<Buffer>,

    pub transforms: Vec<Id<Buffer>>,
    pub materials: Vec<Id<Buffer>>,
    /// Index into `materials` of each composite group's `GroupParams` buffer (`None` for the
    /// screen group).
    pub group_materials: Vec<Option<usize>>,
}

impl Buffers {
    pub fn new(
        audio_settings: &AudioSettings,
        scene_data: &SceneData,
        transform_count: usize,
        resources: &Resources,
        stream: &Stream,
        global_parameters: &GlobalParameters,
    ) -> Self {
        Self {
            global: create_storage_buffer(resources, global_parameters.layout()),
            waveform: create_storage_buffer(resources, stream.layout()),
            dft: create_storage_buffer(
                resources,
                DeviceLayout::new_unsized::<shaders::Dft>(audio_settings.dft_bin_count as u64)
                    .unwrap(),
            ),
            bands: create_storage_buffer(
                resources,
                DeviceLayout::new_unsized::<shaders::Bands>(
                    audio_settings.bands_history_length as u64,
                )
                .unwrap(),
            ),
            transforms: (0..transform_count)
                .map(|_| {
                    create_storage_buffer(
                        resources,
                        DeviceLayout::new_sized::<shaders::Transform>(),
                    )
                })
                .collect(),
            materials: scene_data
                .materials
                .iter()
                .map(|material| create_storage_buffer(resources, material.parameters.layout()))
                .collect(),
            group_materials: Vec::new(),
        }
    }

    /// Appends one `GroupParams` buffer per composite group (all groups except the screen group).
    pub fn add_group_materials(&mut self, resources: &Resources, group_count: usize) {
        self.group_materials = vec![None; group_count];
        for group in 1..group_count {
            let id =
                create_storage_buffer(resources, DeviceLayout::new_sized::<shaders::GroupParams>());
            self.group_materials[group] = Some(self.materials.len());
            self.materials.push(id);
        }
    }

    pub fn group_material(&self, group: usize) -> usize {
        self.group_materials[group].expect("the screen group has no material buffer")
    }

    /// Writes the initial scene data into every buffer that is written from the host.
    pub(crate) fn write_initial_data(
        &self,
        audio_settings: &AudioSettings,
        scene_data: &SceneData,
        tcx: &mut TaskContext<'_>,
    ) {
        for (index, material) in scene_data.materials.iter().enumerate() {
            material.parameters.write(self.materials[index], tcx);
        }

        let dft = tcx.write_buffer::<shaders::Dft>(self.dft, ..);
        let periods = 2.0;
        dft.bin_count = audio_settings.dft_bin_count as u32;
        dft.lowest_frequency =
            audio_settings.sample_rate as f32 / audio_settings.sample_count as f32 * periods;
        dft.exp_bins = (audio_settings.dft_bin_count as f32
            / (audio_settings.sample_count as f32 / (2.0 * periods)).log2())
        .floor()
        .into();

        let bands = tcx.write_buffer::<shaders::Bands>(self.bands, ..);
        bands.history_length = audio_settings.bands_history_length as u32;
        bands.history_delta = audio_settings.bands_history_delta.into();
        bands.time_since_push = 0.0.into();
        bands.chrono = Vec4::ZERO.into();
        bands.start = (audio_settings.bands_history_length as u32 - 1).into();
    }
}

fn create_storage_buffer(resources: &Resources, layout: DeviceLayout) -> Id<Buffer> {
    resources
        .create_buffer(
            &BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            &AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            layout,
        )
        .unwrap()
}
