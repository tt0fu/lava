use glam::Vec4;
use std::sync::Arc;
use vulkano::{
    VulkanError,
    buffer::{BufferCreateInfo, BufferUsage},
    device::{Device, Queue},
    format::Format,
    image::{
        ImageAspects, ImageCreateInfo, ImageLayout, ImageSubresourceLayers, ImageType, ImageUsage,
        sampler::{Filter, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::ImageViewCreateInfo,
    },
    instance::Instance,
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
    pipeline::graphics::viewport::Viewport,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use vulkano_taskgraph::{
    Id, QueueFamilyType,
    command_buffer::{BufferImageCopy, CopyBufferToImageInfo},
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
        parameters::ImageIds,
        scene_data::{ResolvedPanel, SceneData},
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
    pub viewport: Viewport,
    pub recreate_swapchain: bool,
    pub rewrite_transforms: bool,
    pub task_graph: ExecutableTaskGraph<Self>,
    pub virtual_swapchain_id: Id<Swapchain>,
    pub global_parameters: GlobalParameters,
    pub stream: Arc<Stream>,

    pub scene_data: Arc<SceneData>,
    /// The scene resolved against the current window size.
    pub resolved: Vec<ResolvedPanel>,

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

        let global_parameters = GlobalParameters::new();

        let resolved = scene_data.resolve(glam::vec2(
            window_size.width as f32,
            window_size.height as f32,
        ));

        let buffers = Buffers::new(
            audio_settings,
            scene_data,
            resolved.len(),
            resources,
            stream,
            &global_parameters,
        );

        if audio_settings.dft_bin_count > 8192 {
            panic!(
                "dft bin count too high: {}, highest supported is 8192",
                audio_settings.dft_bin_count
            );
        }

        let mut staging_buffer_ids = Vec::new();
        let mut image_ids = Vec::new();
        for image in &scene_data.images {
            let staging_buffer_id = resources
                .create_buffer(
                    &BufferCreateInfo {
                        usage: BufferUsage::TRANSFER_SRC,
                        ..Default::default()
                    },
                    &AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_HOST
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    DeviceLayout::from_size_alignment(image.data.len() as u64, 1).unwrap(),
                )
                .unwrap();
            let image_id = resources
                .create_image(
                    &ImageCreateInfo {
                        image_type: ImageType::Dim2d,
                        format: Format::R8G8B8A8_UNORM,
                        extent: [image.width, image.height, 1],
                        usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                        ..Default::default()
                    },
                    &AllocationCreateInfo::default(),
                )
                .unwrap();
            staging_buffer_ids.push(staging_buffer_id);
            image_ids.push(image_id);
        }

        let bcx = resources.bindless_context().unwrap();

        let sampler_id = bcx
            .global_set()
            .create_sampler(&SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                max_lod: None,
                ..Default::default()
            })
            .unwrap();

        let sampled_image_ids = image_ids
            .iter()
            .map(|&image_id| {
                let image_state = resources.image(image_id);
                let create_info = ImageViewCreateInfo::from_image(image_state.image());
                bcx.global_set()
                    .create_sampled_image(
                        image_id,
                        &create_info,
                        ImageLayout::ShaderReadOnlyOptimal,
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let image_bindings: ImageIds = scene_data
            .images
            .iter()
            .map(|image| image.name.clone())
            .zip(sampled_image_ids.iter().copied())
            .collect();
        for material in &scene_data.materials {
            material
                .parameters
                .resolve_images(&image_bindings)
                .expect("image parameters were validated while parsing the config");
        }

        unsafe {
            vulkano_taskgraph::execute(
                queue,
                resources,
                flight_id,
                |cbf, tcx| {
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

                    for (image, &staging_buffer_id) in
                        scene_data.images.iter().zip(&staging_buffer_ids)
                    {
                        tcx.write_buffer::<[u8]>(staging_buffer_id, ..)
                            .copy_from_slice(&image.data);
                    }
                    for (i, image) in scene_data.images.iter().enumerate() {
                        cbf.copy_buffer_to_image(&CopyBufferToImageInfo {
                            src_buffer: staging_buffer_ids[i],
                            dst_image: image_ids[i],
                            regions: &[BufferImageCopy {
                                image_subresource: ImageSubresourceLayers {
                                    aspects: ImageAspects::COLOR,
                                    ..Default::default()
                                },
                                image_extent: [image.width, image.height, 1],
                                ..Default::default()
                            }],
                            ..Default::default()
                        });
                    }

                    Ok(())
                },
                buffers
                    .materials
                    .iter()
                    .map(|&id| (id, HostAccessType::Write))
                    .chain([
                        (buffers.dft, HostAccessType::Write),
                        (buffers.bands, HostAccessType::Write),
                    ])
                    .chain(
                        staging_buffer_ids
                            .iter()
                            .map(|&id| (id, HostAccessType::Write)),
                    ),
                staging_buffer_ids
                    .iter()
                    .map(|&id| (id, AccessTypes::COPY_TRANSFER_READ)),
                image_ids.iter().map(|&id| {
                    (
                        id,
                        AccessTypes::COPY_TRANSFER_WRITE,
                        ImageLayoutType::Optimal,
                    )
                }),
            )
        }
        .unwrap();

        task_graph.add_host_buffer_access(buffers.global, HostAccessType::Write);
        task_graph.add_host_buffer_access(buffers.waveform, HostAccessType::Write);
        buffers
            .transforms
            .iter()
            .for_each(|&id| task_graph.add_host_buffer_access(id, HostAccessType::Write));

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
            .buffer_access(
                buffers.dft,
                AccessTypes::COMPUTE_SHADER_STORAGE_READ
                    | AccessTypes::COMPUTE_SHADER_STORAGE_WRITE,
            )
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
            .buffer_access(
                buffers.bands,
                AccessTypes::COMPUTE_SHADER_STORAGE_READ
                    | AccessTypes::COMPUTE_SHADER_STORAGE_WRITE,
            )
            .buffer_access(
                buffers.waveform,
                AccessTypes::COMPUTE_SHADER_STORAGE_READ
                    | AccessTypes::COMPUTE_SHADER_STORAGE_WRITE,
            )
            .build();

        let mut render_node = task_graph.create_task_node(
            "render",
            QueueFamilyType::Graphics,
            RenderTask::new(
                resources,
                queue,
                flight_id,
                scene_data,
                virtual_swapchain_id,
            ),
        );
        render_node
            .framebuffer(virtual_framebuffer_id)
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
            .buffer_access(buffers.bands, AccessTypes::FRAGMENT_SHADER_STORAGE_READ);
        for &transform_id in &buffers.transforms {
            render_node.buffer_access(
                transform_id,
                AccessTypes::VERTEX_SHADER_STORAGE_READ | AccessTypes::FRAGMENT_SHADER_STORAGE_READ,
            );
        }
        for &material_id in &buffers.materials {
            render_node.buffer_access(material_id, AccessTypes::FRAGMENT_SHADER_STORAGE_READ);
        }
        for &image_id in &image_ids {
            render_node.image_access(
                image_id,
                AccessTypes::FRAGMENT_SHADER_SAMPLED_READ,
                ImageLayoutType::Optimal,
            );
        }
        let render_node_id = render_node.build();

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
            .create_render_data(
                device,
                bcx,
                &storage_buffers,
                scene_data,
                &resolved,
                &subpass,
                sampler_id,
            );

        let recreate_swapchain = false;
        let rewrite_transforms = true;
        RenderContext {
            window,
            swapchain_id,
            viewport,
            recreate_swapchain,
            rewrite_transforms,
            task_graph,
            virtual_swapchain_id,
            buffers,
            global_parameters,
            stream: stream.clone(),
            scene_data: scene_data.clone(),
            resolved,
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

        self.viewport.extent = window_size.into();
        self.resolved = self.scene_data.resolve(glam::vec2(
            window_size.width as f32,
            window_size.height as f32,
        ));
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
