use std::sync::Arc;

use vulkano::{
    VulkanError,
    device::{Device, Queue},
    instance::Instance,
    swapchain::{Swapchain, SwapchainCreateInfo},
};
use vulkano_taskgraph::{
    Id,
    graph::{ExecutableTaskGraph, ExecuteError, ResourceMap, TaskGraph},
    resource::{Flight, HostAccessType, Resources},
};
use winit::event_loop::ActiveEventLoop;

use crate::{
    audio::{audio_settings::AudioSettings, stream::Stream},
    video::{
        buffers::Buffers,
        geometry::ResolvedScene,
        global_parameters::GlobalParameters,
        group_targets::GroupTargets,
        pipeline::{create_shared_pipeline_layout, load_group_shader, load_vertex_shader},
        render_graph::{self, RenderGraphInputs},
        sampled_image::create_sampler,
        scene_data::SceneData,
        scene_upload,
        storage_buffers::StorageBuffers,
        vertex_buffer::create_vertex_buffer,
        window::WindowState,
    },
};

pub struct RenderContext {
    pub window_state: WindowState,
    pub task_graph: ExecutableTaskGraph<Self>,
    pub virtual_swapchain_id: Id<Swapchain>,
    pub global_parameters: GlobalParameters,
    pub stream: Arc<Stream>,

    pub scene_data: Arc<SceneData>,
    /// The scene resolved against the current window size.
    pub resolved: ResolvedScene,
    pub group_targets: GroupTargets,
    pub buffers: Buffers,

    pub rewrite_transforms: bool,
}

impl RenderContext {
    pub fn new(
        event_loop: &ActiveEventLoop,
        instance: &Arc<Instance>,
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        resources: &Arc<Resources>,
        flight_id: Id<Flight>,
        scene_data: &Arc<SceneData>,
        audio_settings: &Arc<AudioSettings>,
        stream: &Arc<Stream>,
    ) -> Self {
        let window_state = WindowState::new(event_loop, instance, device, resources);
        let resolved = ResolvedScene::new(scene_data, window_state.size());

        let transform_count = resolved.panels.len() + resolved.groups.len() - 1;
        let global_parameters = GlobalParameters::new();
        let mut buffers = Buffers::new(
            audio_settings,
            scene_data,
            transform_count,
            resources,
            stream,
            &global_parameters,
        );
        buffers.add_group_materials(resources, resolved.groups.len());

        if audio_settings.dft_bin_count > 8192 {
            panic!(
                "dft bin count too high: {}, highest supported is 8192",
                audio_settings.dft_bin_count
            );
        }

        let bcx = resources.bindless_context().unwrap();
        let image_ids = scene_upload::upload_scene(
            scene_data,
            &buffers,
            audio_settings,
            resources,
            queue,
            flight_id,
            bcx,
        );

        let mut task_graph = TaskGraph::new(resources);
        let virtual_swapchain_id = task_graph.add_swapchain(&SwapchainCreateInfo {
            image_format: window_state.format,
            ..Default::default()
        });
        let group_targets = GroupTargets::new(
            &mut task_graph,
            resources,
            bcx,
            &resolved,
            window_state.format,
        );

        for &id in &buffers.transforms {
            task_graph.add_host_buffer_access(id, HostAccessType::Write);
        }
        for group in 1..resolved.groups.len() {
            let id = buffers.materials[buffers.group_material(group)];
            task_graph.add_host_buffer_access(id, HostAccessType::Write);
        }
        task_graph.add_host_buffer_access(buffers.global, HostAccessType::Write);
        task_graph.add_host_buffer_access(buffers.waveform, HostAccessType::Write);

        let vertex_shader = load_vertex_shader(device);
        let group_shader = load_group_shader(device);
        let layout = create_shared_pipeline_layout(bcx, scene_data, &vertex_shader, &group_shader);
        let vertex_buffer_id = create_vertex_buffer(resources, queue, flight_id);
        let storage_buffers = StorageBuffers::new(bcx, &buffers);
        let sampler_id = create_sampler(bcx);

        let inputs = RenderGraphInputs {
            device,
            queue,
            flight_id,
            bcx,
            audio_settings,
            scene_data,
            scene_image_ids: &image_ids,
            resolved: &resolved,
            buffers: &buffers,
            storage_buffers: &storage_buffers,
            group_targets: &group_targets,
            vertex_buffer_id,
            layout: &layout,
            group_shader: &group_shader,
            sampler_id,
            virtual_swapchain_id,
        };
        let task_graph = render_graph::build(task_graph, &inputs);

        RenderContext {
            window_state,
            task_graph,
            virtual_swapchain_id,
            global_parameters,
            stream: stream.clone(),
            scene_data: scene_data.clone(),
            resolved,
            group_targets,
            buffers,
            rewrite_transforms: true,
        }
    }

    fn recreate_swapchain(&mut self, resources: &Arc<Resources>) {
        if !self.window_state.recreate_requested {
            return;
        }
        self.window_state.recreate_swapchain(resources);
        self.resolved = ResolvedScene::new(&self.scene_data, self.window_state.size());
        if let Some(bcx) = resources.bindless_context() {
            self.group_targets
                .recreate(resources, bcx, &self.resolved, self.window_state.format);
        }
        self.rewrite_transforms = true;
    }

    pub fn redraw(&mut self, resources: &Arc<Resources>, flight_id: Id<Flight>) {
        self.global_parameters.update();
        self.recreate_swapchain(resources);

        let flight = resources.flight(flight_id);
        flight.wait(None).unwrap();

        let mut resource_map = ResourceMap::new(&self.task_graph).unwrap();
        resource_map
            .insert(self.virtual_swapchain_id, self.window_state.swapchain_id)
            .unwrap();
        for (virtual_id, physical_id) in self.group_targets.physical_mappings() {
            resource_map.insert(virtual_id, physical_id).unwrap();
        }

        match unsafe {
            self.task_graph.execute(resource_map, self, || {
                self.window_state.window.pre_present_notify()
            })
        } {
            Ok(()) => {}
            Err(ExecuteError::Swapchain {
                error: VulkanError::OutOfDate,
                ..
            }) => {
                self.window_state.recreate_requested = true;
            }
            Err(e) => {
                panic!("Failed to execute next frame: {e:?}");
            }
        }
    }
}
