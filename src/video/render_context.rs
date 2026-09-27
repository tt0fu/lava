use glam::Vec4;
use std::sync::Arc;
use vulkano::{
    VulkanError,
    device::{Device, Queue},
    format::Format,
    image::{Image, ImageCreateInfo, ImageType, ImageUsage},
    instance::Instance,
    memory::allocator::AllocationCreateInfo,
    pipeline::graphics::viewport::Viewport,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use vulkano_taskgraph::{
    Id, QueueFamilyType,
    graph::{AttachmentInfo, CompileInfo, ExecutableTaskGraph, ExecuteError, TaskGraph},
    resource::{AccessTypes, Flight, HostAccessType, ImageLayoutType, Resources},
    resource_map,
};
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::{
    audio::{audio_settings::AudioSettings, stream::Stream},
    video::{
        app::MIN_SWAPCHAIN_IMAGES,
        buffers::{Buffers, StorageBuffers},
        global_parameters::GlobalParameters,
        scene_data::SceneData,
        shaders,
        tasks::{
            analysis_task::AnalysisTask, dft_task::DftTask, render_task::RenderTask,
            write_task::WriteTask,
        },
    },
};

pub struct RenderContext {
    pub window: Arc<Window>,
    pub swapchain_id: Id<Swapchain>,
    pub depth_buffer_id: Id<Image>,
    pub viewport: Viewport,
    pub recreate_swapchain: bool,
    pub rewrite_transforms: bool,
    pub task_graph: ExecutableTaskGraph<Self>,
    pub virtual_swapchain_id: Id<Swapchain>,
    pub virtual_depth_buffer_id: Id<Image>,
    pub global_parameters: GlobalParameters,
    pub stream: Arc<Stream>,

    pub buffers: Buffers,
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
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let surface = Surface::from_window(instance, &window).unwrap();
        let window_size = window.inner_size();
        let swapchain_format;
        let swapchain_id = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, &Default::default())
                .unwrap();
            (swapchain_format, _) = device
                .physical_device()
                .surface_formats(&surface, &Default::default())
                .unwrap()[0];
            resources
                .create_swapchain(
                    &surface,
                    &SwapchainCreateInfo {
                        min_image_count: surface_capabilities
                            .min_image_count
                            .max(MIN_SWAPCHAIN_IMAGES),
                        image_format: swapchain_format,
                        image_extent: window_size.into(),
                        image_usage: ImageUsage::COLOR_ATTACHMENT,
                        composite_alpha: surface_capabilities
                            .supported_composite_alpha
                            .into_iter()
                            .next()
                            .unwrap(),
                        ..Default::default()
                    },
                )
                .unwrap()
        };
        let depth_buffer_create_info = ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::D16_UNORM,
            extent: [window_size.width, window_size.height, 1],
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
            ..Default::default()
        };
        let depth_buffer_id = resources
            .create_image(&depth_buffer_create_info, &AllocationCreateInfo::default())
            .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window_size.into(),
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let mut task_graph = TaskGraph::new(resources);
        let virtual_swapchain_id = task_graph.add_swapchain(&SwapchainCreateInfo {
            image_format: swapchain_format,
            ..Default::default()
        });
        let virtual_framebuffer_id = task_graph.add_framebuffer();
        let virtual_depth_buffer_id = task_graph.add_image(&depth_buffer_create_info);

        let global_parameters = GlobalParameters::new();

        let buffers = Buffers::new(
            audio_settings,
            scene_data,
            resources,
            stream,
            &global_parameters,
        );

        if audio_settings.dft_bin_count > 8192 {
            panic!("dft bin count too high: {}, highest supported is 8192", audio_settings.dft_bin_count);
        }

        unsafe {
            vulkano_taskgraph::execute(
                queue,
                resources,
                flight_id,
                |_cbf, tcx| {
                    for i in 0..scene_data.materials.len() {
                        scene_data.materials[i]
                            .parameters
                            .write(buffers.materials[i], tcx);
                    }
                    let dft_guard = tcx.write_buffer::<shaders::Dft>(buffers.dft, ..);
                    dft_guard.bin_count = audio_settings.dft_bin_count as u32;
                    let periods = 2.0;
                    dft_guard.lowest_frequency = audio_settings.sample_rate as f32
                        / audio_settings.sample_count as f32
                        * periods;
                    dft_guard.exp_bins = (audio_settings.dft_bin_count as f32
                        / (audio_settings.sample_count as f32 / (2.0 * periods)).log2())
                    .floor()
                    .into();
                    let bands_guard = tcx.write_buffer::<shaders::Bands>(buffers.bands, ..);
                    bands_guard.history_length = audio_settings.bands_history_length as u32;
                    bands_guard.history_delta = audio_settings.bands_history_delta.into();
                    bands_guard.time_since_push = 0.0.into();
                    bands_guard.chrono = Vec4::ZERO.into();
                    bands_guard.start = (audio_settings.bands_history_length as u32 - 1).into();

                    Ok(())
                },
                buffers
                    .materials
                    .iter()
                    .map(|&id| (id, HostAccessType::Write))
                    .chain([
                        (buffers.dft, HostAccessType::Write),
                        (buffers.bands, HostAccessType::Write),
                    ]),
                [],
                [],
            )
        }
        .unwrap();

        task_graph.add_host_buffer_access(buffers.global, HostAccessType::Write);
        task_graph.add_host_buffer_access(buffers.waveform, HostAccessType::Write);
        buffers
            .transforms
            .iter()
            .for_each(|&id| task_graph.add_host_buffer_access(id, HostAccessType::Write));

        let bcx = resources.bindless_context().unwrap();

        let storage_buffers = StorageBuffers::new(bcx, &buffers);

        let write_node_id = task_graph
            .create_task_node("Write", QueueFamilyType::Compute, WriteTask {})
            .buffer_access(buffers.global, AccessTypes::COMPUTE_SHADER_STORAGE_WRITE)
            .buffer_access(buffers.waveform, AccessTypes::COMPUTE_SHADER_STORAGE_WRITE)
            .build();

        let dft_node_id = task_graph
            .create_task_node(
                "Dft",
                QueueFamilyType::Compute,
                DftTask::new(
                    audio_settings,
                    device,
                    bcx,
                    storage_buffers.compute_push_constants(),
                ),
            )
            .buffer_access(buffers.waveform, AccessTypes::COMPUTE_SHADER_STORAGE_READ)
            .buffer_access(buffers.dft, AccessTypes::COMPUTE_SHADER_STORAGE_WRITE)
            .build();

        let analysis_node_id = task_graph
            .create_task_node(
                "Analysis",
                QueueFamilyType::Compute,
                AnalysisTask::new(
                    audio_settings,
                    device,
                    bcx,
                    storage_buffers.compute_push_constants(),
                ),
            )
            .buffer_access(buffers.global, AccessTypes::COMPUTE_SHADER_STORAGE_READ)
            .buffer_access(buffers.dft, AccessTypes::COMPUTE_SHADER_STORAGE_READ)
            .buffer_access(buffers.bands, AccessTypes::COMPUTE_SHADER_STORAGE_READ)
            .buffer_access(buffers.bands, AccessTypes::COMPUTE_SHADER_STORAGE_WRITE)
            .buffer_access(buffers.waveform, AccessTypes::COMPUTE_SHADER_STORAGE_WRITE)
            .build();

        let render_node_id = task_graph
            .create_task_node(
                "render",
                QueueFamilyType::Graphics,
                RenderTask::new(
                    resources,
                    queue,
                    flight_id,
                    scene_data,
                    virtual_swapchain_id,
                    virtual_depth_buffer_id,
                ),
            )
            .framebuffer(virtual_framebuffer_id)
            .depth_stencil_attachment(
                virtual_depth_buffer_id,
                AccessTypes::DEPTH_STENCIL_ATTACHMENT_READ
                    | AccessTypes::DEPTH_STENCIL_ATTACHMENT_WRITE,
                ImageLayoutType::Optimal,
                &AttachmentInfo {
                    clear: true,
                    ..Default::default()
                },
            )
            .color_attachment(
                virtual_swapchain_id.current_image_id(),
                AccessTypes::COLOR_ATTACHMENT_WRITE,
                ImageLayoutType::Optimal,
                &AttachmentInfo {
                    clear: true,
                    ..Default::default()
                },
            )
            .buffer_access(buffers.global, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
            .buffer_access(buffers.waveform, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
            .buffer_access(buffers.dft, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
            .buffer_access(buffers.bands, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
            .build();

        task_graph.add_edge(write_node_id, dft_node_id).unwrap();
        task_graph.add_edge(dft_node_id, analysis_node_id).unwrap();
        task_graph
            .add_edge(analysis_node_id, render_node_id)
            .unwrap();

        let mut task_graph = unsafe {
            task_graph.compile(&CompileInfo {
                queues: &[queue],
                present_queue: Some(queue),
                flight_id,
                ..Default::default()
            })
        }
        .unwrap();
        let render_node = task_graph.task_node_mut(render_node_id).unwrap();

        let subpass = render_node.subpass().unwrap().clone();

        render_node
            .task_mut()
            .downcast_mut::<RenderTask>()
            .unwrap()
            .create_render_data(device, bcx, &storage_buffers, scene_data, &subpass);

        let recreate_swapchain = false;
        let rewrite_transforms = true;
        RenderContext {
            window,
            swapchain_id,
            depth_buffer_id,
            viewport,
            recreate_swapchain,
            rewrite_transforms,
            task_graph,
            virtual_swapchain_id,
            virtual_depth_buffer_id,
            buffers,
            global_parameters,
            stream: stream.clone(),
        }
    }

    fn recreate_swapchain(&mut self, resources: &Arc<Resources>) {
        if !self.recreate_swapchain {
            return;
        }
        let window_size = self.window.inner_size();
        self.swapchain_id = resources
            .recreate_swapchain(self.swapchain_id, |create_info| SwapchainCreateInfo {
                image_extent: window_size.into(),
                ..*create_info
            })
            .expect("failed to recreate swapchain");

        let mut batch = resources.create_deferred_batch();
        batch.destroy_image(self.depth_buffer_id);
        batch.enqueue();

        self.depth_buffer_id = resources
            .create_image(
                &ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D16_UNORM,
                    extent: [window_size.width, window_size.height, 1],
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    ..Default::default()
                },
                &AllocationCreateInfo::default(),
            )
            .unwrap();
        self.viewport.extent = window_size.into();
        self.recreate_swapchain = false;
    }

    pub fn redraw(&mut self, resources: &Arc<Resources>, flight_id: Id<Flight>) {
        self.global_parameters.update();

        self.rewrite_transforms = self.recreate_swapchain;
        self.recreate_swapchain(resources);

        let flight = resources.flight(flight_id);
        flight.wait(None).unwrap();

        let resource_map = resource_map!(&self.task_graph,
            self.virtual_swapchain_id => self.swapchain_id,
            self.virtual_depth_buffer_id => self.depth_buffer_id
        )
        .unwrap();
        match unsafe {
            self.task_graph
                .execute(resource_map, self, || self.window.pre_present_notify())
        } {
            Ok(()) => {}
            Err(ExecuteError::Swapchain {
                error: VulkanError::OutOfDate,
                ..
            }) => {
                self.recreate_swapchain = true;
            }
            Err(e) => {
                panic!("Failed to execute next frame: {e:?}");
            }
        }
    }
}
