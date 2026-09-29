use std::sync::Arc;

use vulkano::{
    buffer::Buffer,
    device::{Device, Queue},
    image::Image,
    pipeline::PipelineLayout,
    shader::EntryPoint,
    swapchain::Swapchain,
};
use vulkano_taskgraph::{
    Id,
    descriptor_set::{BindlessContext, SamplerId},
    graph::{CompileInfo, ExecutableTaskGraph, TaskGraph},
    resource::Flight,
};

use crate::{
    audio::audio_settings::AudioSettings,
    video::{
        buffers::Buffers,
        geometry::ResolvedScene,
        group_targets::GroupTargets,
        render_context::RenderContext,
        render_nodes::{add_compute_nodes, add_group_nodes},
        scene_data::SceneData,
        storage_buffers::StorageBuffers,
        tasks::{draw::RenderData, render_task::RenderTask},
    },
};

pub struct RenderGraphInputs<'a> {
    pub device: &'a Arc<Device>,
    pub queue: &'a Arc<Queue>,
    pub flight_id: Id<Flight>,
    pub bcx: &'a BindlessContext,
    pub audio_settings: &'a Arc<AudioSettings>,
    pub scene_data: &'a Arc<SceneData>,
    pub scene_image_ids: &'a [Id<Image>],
    pub resolved: &'a ResolvedScene,
    pub buffers: &'a Buffers,
    pub storage_buffers: &'a StorageBuffers,
    pub group_targets: &'a GroupTargets,
    pub vertex_buffer_id: Id<Buffer>,
    pub layout: &'a Arc<PipelineLayout>,
    pub group_shader: &'a EntryPoint,
    pub sampler_id: SamplerId,
    pub virtual_swapchain_id: Id<Swapchain>,
}

pub fn build(
    mut task_graph: TaskGraph<RenderContext>,
    inputs: &RenderGraphInputs<'_>,
) -> ExecutableTaskGraph<RenderContext> {
    let analysis_node = add_compute_nodes(&mut task_graph, inputs);
    let group_nodes = add_group_nodes(&mut task_graph, inputs, analysis_node);

    let mut executable = unsafe {
        task_graph.compile(&CompileInfo {
            queues: &[inputs.queue],
            present_queue: Some(inputs.queue),
            flight_id: inputs.flight_id,
            ..Default::default()
        })
    }
    .unwrap();

    for node_id in &group_nodes {
        let node = executable.task_node_mut(*node_id).unwrap();
        let subpass = node.subpass().unwrap().clone();
        let task = node.task_mut().downcast_mut::<RenderTask>().unwrap();
        task.render_data = Some(RenderData::new(
            inputs.device,
            inputs.layout,
            inputs.buffers,
            inputs.storage_buffers,
            inputs.scene_data,
            inputs.resolved,
            task.group,
            inputs.group_shader,
            &subpass,
            inputs.sampler_id,
        ));
    }

    executable
}
