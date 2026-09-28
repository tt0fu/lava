use glam::{Vec2, Vec4};
use std::sync::Arc;
use vulkano::{
    VulkanError,
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::{Device, Queue},
    format::Format,
    image::{
        Image, ImageAspects, ImageCreateInfo, ImageLayout, ImageSubresourceLayers, ImageType,
        ImageUsage,
        sampler::{Filter, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::ImageViewCreateInfo,
    },
    instance::Instance,
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
    pipeline::{PipelineShaderStageCreateInfo, graphics::viewport::Viewport},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use vulkano_taskgraph::{
    Id, QueueFamilyType,
    command_buffer::{BufferImageCopy, CopyBufferToImageInfo},
    descriptor_set::SampledImageId,
    graph::{
        AttachmentInfo, CompileInfo, ExecutableTaskGraph, ExecuteError, ResourceMap, TaskGraph,
    },
    resource::{AccessTypes, Flight, HostAccessType, ImageLayoutType, Resources},
};

use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::{
    audio::{audio_settings::AudioSettings, stream::Stream},
    video::{
        app::MIN_SWAPCHAIN_IMAGES,
        buffers::{Buffers, StorageBuffers},
        geometry::{ResolvedChild, ResolvedScene},
        global_parameters::GlobalParameters,
        model::{MyVertex, VERTICES},
        parameters::ImageIds,
        scene_data::SceneData,
        shaders,
        tasks::{
            analysis_task::AnalysisTask,
            dft_task::DftTask,
            render_task::{RenderTask, Target},
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

    pub queue: Arc<Queue>,
    pub flight_id: Id<Flight>,

    pub scene_data: Arc<SceneData>,
    /// The scene resolved against the current window size.
    pub resolved: ResolvedScene,
    /// Physical offscreen target per group (`None` for the screen group).
    pub target_physical: Vec<Option<Id<Image>>>,
    /// Virtual offscreen target per group (`None` for the screen group).
    pub virtual_target: Vec<Option<Id<Image>>>,
    /// Bindless sampled image per group target (`None` for the screen group).
    pub target_sampled: Vec<Option<SampledImageId>>,
    /// Index into `buffers.materials` of each group's `GroupParams` buffer (`None` for the screen).
    pub group_material: Vec<Option<usize>>,

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
        let screen_size = Vec2::new(window_size.width as f32, window_size.height as f32);
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

        let resolved = ResolvedScene::new(scene_data, screen_size);

        // Transform buffers: one per panel plus one per composite group (all groups except the
        // screen group).
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

        // One `GroupParams` buffer per composite group, appended after the config materials.
        let group_params_layout = DeviceLayout::new_sized::<shaders::GroupParams>();
        let mut group_material = vec![None; resolved.groups.len()];
        for group in 1..resolved.groups.len() {
            let id = resources
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
                    group_params_layout,
                )
                .unwrap();
            group_material[group] = Some(buffers.materials.len());
            buffers.materials.push(id);
        }

        let mut task_graph = TaskGraph::new(resources);
        let virtual_swapchain_id = task_graph.add_swapchain(&SwapchainCreateInfo {
            image_format: swapchain_format,
            ..Default::default()
        });

        // Virtual offscreen targets.
        let mut virtual_target = vec![None; resolved.groups.len()];
        for group in 1..resolved.groups.len() {
            let size = resolved.groups[group].aabb_size;
            virtual_target[group] = Some(task_graph.add_image(&ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: swapchain_format,
                extent: [size.x.max(1.0) as u32, size.y.max(1.0) as u32, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            }));
        }

        if audio_settings.dft_bin_count > 8192 {
            panic!(
                "dft bin count too high: {}, highest supported is 8192",
                audio_settings.dft_bin_count
            );
        }

        // Config image staging buffers and images.
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

        // Offscreen group targets.
        let mut target_physical = vec![None; resolved.groups.len()];
        let mut target_sampled = vec![None; resolved.groups.len()];
        for group in 1..resolved.groups.len() {
            let size = resolved.groups[group].aabb_size;
            let image_id = create_target(resources, swapchain_format, size);
            let image_state = resources.image(image_id);
            let create_info = ImageViewCreateInfo::from_image(image_state.image());
            target_sampled[group] = Some(
                bcx.global_set()
                    .create_sampled_image(
                        image_id,
                        &create_info,
                        ImageLayout::ShaderReadOnlyOptimal,
                    )
                    .unwrap(),
            );
            target_physical[group] = Some(image_id);
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
        // The group params buffers are host-written by the render task (like the transforms).
        for group in 1..resolved.groups.len() {
            let material = group_material[group].unwrap();
            task_graph.add_host_buffer_access(buffers.materials[material], HostAccessType::Write);
        }

        let storage_buffers = StorageBuffers::new(bcx, &buffers);

        // Shared pipeline layout (union of all fragment shaders plus the composite shader).
        let vertex_shader = unsafe { shaders::load_vertex(device) }
            .unwrap()
            .entry_point("main")
            .unwrap();
        let group_shader = unsafe { shaders::load_group(device) }
            .unwrap()
            .entry_point("main")
            .unwrap();
        let all_stages = std::iter::once(PipelineShaderStageCreateInfo::new(&vertex_shader))
            .chain(
                scene_data
                    .shaders
                    .iter()
                    .map(PipelineShaderStageCreateInfo::new),
            )
            .chain(std::iter::once(PipelineShaderStageCreateInfo::new(
                &group_shader,
            )))
            .collect::<Vec<_>>();
        let layout = bcx.pipeline_layout_from_stages(&all_stages).unwrap();

        // Vertex buffer (shared by every group).
        let vertex_buffer_id = create_vertex_buffer(resources, queue, flight_id);

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
        task_graph.add_edge(write_node_id, dft_node_id).unwrap();
        task_graph.add_edge(dft_node_id, analysis_node_id).unwrap();

        // One render node per group.
        let mut group_node_ids = vec![None; resolved.groups.len()];
        for group in (0..resolved.groups.len()).rev() {
            let info = &resolved.groups[group];
            let target = if group == resolved.root {
                Target::Swapchain(virtual_swapchain_id)
            } else {
                Target::Image(virtual_target[group].unwrap())
            };
            let background: [f32; 4] = info.background.into();
            let framebuffer_id = task_graph.add_framebuffer();
            let mut node = task_graph.create_task_node(
                "Group",
                QueueFamilyType::Graphics,
                RenderTask::new(vertex_buffer_id, group, target, background),
            );
            node.framebuffer(framebuffer_id).color_attachment(
                if group == resolved.root {
                    virtual_swapchain_id.current_image_id()
                } else {
                    virtual_target[group].unwrap()
                },
                AccessTypes::COLOR_ATTACHMENT_READ | AccessTypes::COLOR_ATTACHMENT_WRITE,
                ImageLayoutType::Optimal,
                &AttachmentInfo {
                    clear: true,
                    ..Default::default()
                },
            );
            // Reads.
            for child in &info.children {
                let (material, transform) = match child {
                    ResolvedChild::Panel(i) => (resolved.panels[*i].material, *i),
                    ResolvedChild::Group(g) => {
                        (group_material[*g].unwrap(), resolved.panels.len() + (g - 1))
                    }
                };
                node.buffer_access(
                    buffers.transforms[transform],
                    AccessTypes::VERTEX_SHADER_STORAGE_READ
                        | AccessTypes::FRAGMENT_SHADER_STORAGE_READ,
                );
                node.buffer_access(
                    buffers.materials[material],
                    AccessTypes::FRAGMENT_SHADER_STORAGE_READ,
                );
            }
            node.buffer_access(buffers.global, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
                .buffer_access(buffers.waveform, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
                .buffer_access(buffers.dft, AccessTypes::FRAGMENT_SHADER_STORAGE_READ)
                .buffer_access(buffers.bands, AccessTypes::FRAGMENT_SHADER_STORAGE_READ);
            for &image_id in &image_ids {
                node.image_access(
                    image_id,
                    AccessTypes::FRAGMENT_SHADER_SAMPLED_READ,
                    ImageLayoutType::Optimal,
                );
            }
            for child in &info.children {
                if let ResolvedChild::Group(g) = child {
                    node.image_access(
                        virtual_target[*g].unwrap(),
                        AccessTypes::FRAGMENT_SHADER_SAMPLED_READ,
                        ImageLayoutType::Optimal,
                    );
                }
            }
            let node_id = node.build();
            group_node_ids[group] = Some(node_id);
            task_graph.add_edge(analysis_node_id, node_id).unwrap();
            for child in &info.children {
                if let ResolvedChild::Group(g) = child {
                    task_graph
                        .add_edge(group_node_ids[*g].unwrap(), node_id)
                        .unwrap();
                }
            }
        }

        let mut task_graph = unsafe {
            task_graph.compile(&CompileInfo {
                queues: &[queue],
                present_queue: Some(queue),
                flight_id,
                ..Default::default()
            })
        }
        .unwrap();

        for group in 0..resolved.groups.len() {
            let node = task_graph
                .task_node_mut(group_node_ids[group].unwrap())
                .unwrap();
            let subpass = node.subpass().unwrap().clone();
            node.task_mut()
                .downcast_mut::<RenderTask>()
                .unwrap()
                .create_render_data(
                    device,
                    &layout,
                    &storage_buffers,
                    scene_data,
                    &resolved,
                    &group_shader,
                    &subpass,
                    sampler_id,
                );
        }

        RenderContext {
            window,
            swapchain_id,
            viewport,
            recreate_swapchain: false,
            rewrite_transforms: true,
            task_graph,
            virtual_swapchain_id,
            global_parameters,
            stream: stream.clone(),
            queue: queue.clone(),
            flight_id,
            scene_data: scene_data.clone(),
            resolved,
            target_physical,
            virtual_target,
            target_sampled,
            group_material,
            buffers,
        }
    }

    fn recreate_swapchain(&mut self, resources: &Arc<Resources>) {
        if !self.recreate_swapchain {
            return;
        }
        let window_size = self.window.inner_size();
        let screen_size = Vec2::new(window_size.width as f32, window_size.height as f32);
        self.swapchain_id = resources
            .recreate_swapchain(self.swapchain_id, |create_info| SwapchainCreateInfo {
                image_extent: window_size.into(),
                ..*create_info
            })
            .expect("failed to recreate swapchain");

        self.resolved = ResolvedScene::new(&self.scene_data, screen_size);

        // Recreate the offscreen targets and re-register their sampled images.
        if let Some(bcx) = resources.bindless_context() {
            let format = resources
                .swapchain(self.swapchain_id)
                .images()
                .first()
                .map(|image| image.format())
                .unwrap_or(Format::R8G8B8A8_UNORM);
            let mut batch = resources.create_deferred_batch();
            for group in 1..self.resolved.groups.len() {
                if let Some(old) = self.target_physical[group] {
                    batch.destroy_image(old);
                }
            }
            batch.enqueue();

            for group in 1..self.resolved.groups.len() {
                let size = self.resolved.groups[group].aabb_size;
                let image_id = create_target(resources, format, size);
                let image_state = resources.image(image_id);
                let create_info = ImageViewCreateInfo::from_image(image_state.image());
                self.target_sampled[group] = Some(
                    bcx.global_set()
                        .create_sampled_image(
                            image_id,
                            &create_info,
                            ImageLayout::ShaderReadOnlyOptimal,
                        )
                        .unwrap(),
                );
                self.target_physical[group] = Some(image_id);
            }
        }

        self.viewport.extent = window_size.into();
        self.rewrite_transforms = true;
        self.recreate_swapchain = false;
    }

    pub fn redraw(&mut self, resources: &Arc<Resources>, flight_id: Id<Flight>) {
        self.global_parameters.update();

        self.recreate_swapchain(resources);

        let flight = resources.flight(flight_id);
        flight.wait(None).unwrap();

        let mut resource_map = ResourceMap::new(&self.task_graph).unwrap();
        resource_map
            .insert(self.virtual_swapchain_id, self.swapchain_id)
            .unwrap();
        for group in 1..self.resolved.groups.len() {
            resource_map
                .insert(
                    self.virtual_target[group].unwrap(),
                    self.target_physical[group].unwrap(),
                )
                .unwrap();
        }

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

fn create_target(resources: &Arc<Resources>, format: Format, size: Vec2) -> Id<Image> {
    resources
        .create_image(
            &ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format,
                extent: [size.x.max(1.0) as u32, size.y.max(1.0) as u32, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            &AllocationCreateInfo::default(),
        )
        .unwrap()
}

fn create_vertex_buffer(
    resources: &Arc<Resources>,
    queue: &Arc<Queue>,
    flight_id: Id<Flight>,
) -> Id<Buffer> {
    let vertex_buffer_id = resources
        .create_buffer(
            &BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            &AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            DeviceLayout::for_value(VERTICES.as_slice()).unwrap(),
        )
        .unwrap();
    unsafe {
        vulkano_taskgraph::execute(
            queue,
            resources,
            flight_id,
            |_cbf, tcx| {
                tcx.try_write_buffer::<[MyVertex]>(vertex_buffer_id, ..)?
                    .copy_from_slice(&VERTICES);
                Ok(())
            },
            [(vertex_buffer_id, HostAccessType::Write)],
            [],
            [],
        )
    }
    .unwrap();
    vertex_buffer_id
}
