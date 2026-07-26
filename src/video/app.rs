use glam::vec3;
use std::sync::Arc;
use vulkano::{
    VulkanError, VulkanLibrary,
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    format::Format,
    image::{Image, ImageCreateInfo, ImageType, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
    pipeline::graphics::viewport::Viewport,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use vulkano_taskgraph::{
    Id, QueueFamilyType,
    descriptor_set::BindlessContext,
    graph::{AttachmentInfo, CompileInfo, ExecutableTaskGraph, ExecuteError, TaskGraph},
    resource::{
        AccessTypes, Flight, HostAccessType, ImageLayoutType, Resources, ResourcesCreateInfo,
    },
    resource_map,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    audio::stream::Stream,
    stats::frame_timer::FrameTimer,
    video::{
        audio_settings::AudioSettings,
        global_parameters::GlobalParameters,
        material_parameters::{BandsParameters, SpectrogramParameters, WaveformParameters},
        parameters::Layout,
        render_task::RenderTask,
        scene_data::{Material, Panel, SceneData},
        shaders,
        transform::Transform,
    },
};

const MAX_FRAMES_IN_FLIGHT: u32 = 2;
const MIN_SWAPCHAIN_IMAGES: u32 = MAX_FRAMES_IN_FLIGHT + 1;

pub struct Buffers {
    pub global: Id<Buffer>,
    pub waveform: Id<Buffer>,
    pub dft: Id<Buffer>,

    pub transforms: Vec<Id<Buffer>>,
    pub materials: Vec<Id<Buffer>>,
}

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

pub struct App {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub resources: Arc<Resources>,
    pub flight_id: Id<Flight>,

    pub scene_data: Arc<SceneData>,
    pub audio_settings: Arc<AudioSettings>,

    pub rcx: Option<RenderContext>,

    pub stream: Arc<Stream>,

    pub frame_timer: FrameTimer,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>, audio_settings: AudioSettings) -> Self {
        let library = unsafe { VulkanLibrary::new() }.unwrap();
        let required_extensions = Surface::required_extensions(event_loop);
        let instance = Instance::new(
            &library,
            &InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: &required_extensions,
                ..Default::default()
            },
        )
        .unwrap();
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, event_loop)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("no suitable physical device found");
        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );
        let (device, mut queues) = Device::new(
            &physical_device,
            &DeviceCreateInfo {
                enabled_extensions: &device_extensions,
                enabled_features: &DeviceFeatures {
                    ..BindlessContext::required_features(&instance)
                },
                queue_create_infos: &[QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let scene_data = SceneData {
            shaders: vec![
                unsafe { shaders::load_simple(&device) }
                    .unwrap()
                    .specialize(&[(0, 48000u32.into())])
                    .entry_point("main")
                    .unwrap(),
                unsafe { shaders::load_clock(&device) }
                    .unwrap()
                    .specialize(&[(0, 48000u32.into())])
                    .entry_point("main")
                    .unwrap(),
                unsafe { shaders::load_waveform(&device) }
                    .unwrap()
                    .specialize(&[(0, 48000u32.into())])
                    .entry_point("main")
                    .unwrap(),
                unsafe { shaders::load_spectrogram(&device) }
                    .unwrap()
                    .specialize(&[(0, 48000u32.into())])
                    .entry_point("main")
                    .unwrap(),
                unsafe { shaders::load_bands(&device) }
                    .unwrap()
                    .specialize(&[(0, 48000u32.into())])
                    .entry_point("main")
                    .unwrap(),
            ],
            transforms: vec![Transform::FULLSCREEN],
            materials: vec![
                Material {
                    shader_id: 2,
                    parameters: Box::new(WaveformParameters {
                        col: vec3(1.0, 1.0, 1.0),
                        line_width: 50.0,
                        gain: 1.5,
                    }),
                },
                Material {
                    shader_id: 3,
                    parameters: Box::new(SpectrogramParameters {
                        col: vec3(1.0, 1.0, 1.0),
                        gain: 1.5,
                    }),
                },
                Material {
                    shader_id: 4,
                    parameters: Box::new(BandsParameters {
                        col: vec3(1.0, 1.0, 1.0),
                        gain: 1.0,
                    }),
                },
            ],
            panels: vec![
                Panel {
                    transform_id: 0,
                    material_id: 2,
                    order: 0,
                },
                // Panel {
                //     transform_id: 0,
                //     material_id: 1,
                //     order: 1,
                // },
            ],
            background_color: vec3(0.0, 0.0, 0.0),
        };

        let queue = queues.next().unwrap();
        let resources = Resources::new(
            &device,
            &ResourcesCreateInfo {
                bindless_context: Some(&Default::default()),
                ..Default::default()
            },
        )
        .unwrap();
        let flight_id = resources.create_flight(MAX_FRAMES_IN_FLIGHT).unwrap();
        let rcx = None;
        let stream = Arc::new(
            Stream::new(
                audio_settings.sample_rate,
                audio_settings.channel_count,
                512,
                audio_settings.sample_count,
            )
            .unwrap(),
        );
        App {
            instance,
            device,
            queue,
            resources,
            flight_id,
            scene_data: Arc::new(scene_data),
            audio_settings: Arc::new(audio_settings),
            rcx,
            stream,
            frame_timer: FrameTimer::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let surface = Surface::from_window(&self.instance, &window).unwrap();
        let window_size = window.inner_size();
        let swapchain_format;
        let swapchain_id = {
            let surface_capabilities = self
                .device
                .physical_device()
                .surface_capabilities(&surface, &Default::default())
                .unwrap();
            (swapchain_format, _) = self
                .device
                .physical_device()
                .surface_formats(&surface, &Default::default())
                .unwrap()[0];
            self.resources
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
        let depth_buffer_id = self
            .resources
            .create_image(&depth_buffer_create_info, &AllocationCreateInfo::default())
            .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window_size.into(),
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let mut task_graph = TaskGraph::new(&self.resources);
        let virtual_swapchain_id = task_graph.add_swapchain(&SwapchainCreateInfo {
            image_format: swapchain_format,
            ..Default::default()
        });
        let virtual_framebuffer_id = task_graph.add_framebuffer();
        let virtual_depth_buffer_id = task_graph.add_image(&depth_buffer_create_info);

        let global_parameters = GlobalParameters::new();

        let buffer_create_info = BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        };
        let allocation_create_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        };
        let buffers = Buffers {
            global: self
                .resources
                .create_buffer(
                    &buffer_create_info,
                    &allocation_create_info,
                    global_parameters.layout(),
                )
                .unwrap(),
            waveform: self
                .resources
                .create_buffer(
                    &buffer_create_info,
                    &allocation_create_info,
                    self.stream.layout(),
                )
                .unwrap(),
            dft: self
                .resources
                .create_buffer(
                    &buffer_create_info,
                    &allocation_create_info,
                    DeviceLayout::new_unsized::<shaders::Dft>(self.audio_settings.bin_count as u64)
                        .unwrap(),
                )
                .unwrap(),
            transforms: self
                .scene_data
                .transforms
                .iter()
                .map(|_| {
                    self.resources
                        .create_buffer(
                            &buffer_create_info,
                            &allocation_create_info,
                            DeviceLayout::new_sized::<shaders::Transform>(),
                        )
                        .unwrap()
                })
                .collect(),
            materials: self
                .scene_data
                .materials
                .iter()
                .map(|m| {
                    self.resources
                        .create_buffer(
                            &buffer_create_info,
                            &allocation_create_info,
                            m.parameters.layout(),
                        )
                        .unwrap()
                })
                .collect(),
        };

        unsafe {
            vulkano_taskgraph::execute(
                &self.queue,
                &self.resources,
                self.flight_id,
                |_cbf, tcx| {
                    for i in 0..self.scene_data.materials.len() {
                        self.scene_data.materials[i]
                            .parameters
                            .write(buffers.materials[i], tcx);
                    }
                    let guard = tcx.write_buffer::<shaders::Dft>(buffers.dft, ..);
                    guard.bin_count = self.audio_settings.bin_count as u32;
                    let periods = 2.0;
                    guard.lowest_frequency = self.audio_settings.sample_rate as f32
                        / self.audio_settings.sample_count as f32
                        * periods;
                    guard.exp_bins = (self.audio_settings.bin_count as f32
                        / (self.audio_settings.sample_count as f32 / (2.0 * periods)).log2())
                    .floor()
                    .into();

                    Ok(())
                },
                buffers
                    .materials
                    .iter()
                    .map(|&id| (id, HostAccessType::Write))
                    .chain(std::iter::once((buffers.dft, HostAccessType::Write))),
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

        let render_node_id = task_graph
            .create_task_node(
                "render",
                QueueFamilyType::Graphics,
                RenderTask::new(self, virtual_swapchain_id, virtual_depth_buffer_id),
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
            .build();
        let mut task_graph = unsafe {
            task_graph.compile(&CompileInfo {
                queues: &[&self.queue],
                present_queue: Some(&self.queue),
                flight_id: self.flight_id,
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
            .create_render_data(self, &subpass, &buffers);
        let recreate_swapchain = false;
        let rewrite_transforms = true;
        self.rcx = Some(RenderContext {
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
            stream: self.stream.clone(),
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let rcx = self.rcx.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                self.frame_timer.print_results();
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                rcx.recreate_swapchain = true;
            }
            WindowEvent::RedrawRequested => {
                self.frame_timer.start_frame();

                rcx.global_parameters.update();

                let window_size = rcx.window.inner_size();
                if window_size.width == 0 || window_size.height == 0 {
                    return;
                }
                rcx.rewrite_transforms = rcx.recreate_swapchain;
                if rcx.recreate_swapchain {
                    rcx.swapchain_id = self
                        .resources
                        .recreate_swapchain(rcx.swapchain_id, |create_info| SwapchainCreateInfo {
                            image_extent: window_size.into(),
                            ..*create_info
                        })
                        .expect("failed to recreate swapchain");

                    let mut batch = self.resources.create_deferred_batch();
                    batch.destroy_image(rcx.depth_buffer_id);
                    batch.enqueue();

                    rcx.depth_buffer_id = self
                        .resources
                        .create_image(
                            &ImageCreateInfo {
                                image_type: ImageType::Dim2d,
                                format: Format::D16_UNORM,
                                extent: [window_size.width, window_size.height, 1],
                                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT
                                    | ImageUsage::TRANSIENT_ATTACHMENT,
                                ..Default::default()
                            },
                            &AllocationCreateInfo::default(),
                        )
                        .unwrap();
                    rcx.viewport.extent = window_size.into();
                    rcx.recreate_swapchain = false;
                }

                let flight = self.resources.flight(self.flight_id);
                flight.wait(None).unwrap();

                let resource_map = resource_map!(&rcx.task_graph,
                    rcx.virtual_swapchain_id => rcx.swapchain_id,
                    rcx.virtual_depth_buffer_id => rcx.depth_buffer_id
                )
                .unwrap();
                match unsafe {
                    rcx.task_graph
                        .execute(resource_map, rcx, || rcx.window.pre_present_notify())
                } {
                    Ok(()) => {}
                    Err(ExecuteError::Swapchain {
                        error: VulkanError::OutOfDate,
                        ..
                    }) => {
                        rcx.recreate_swapchain = true;
                    }
                    Err(e) => {
                        panic!("Failed to execute next frame: {e:?}");
                    }
                }
                self.frame_timer.end_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let rcx = self.rcx.as_mut().unwrap();
        rcx.window.request_redraw();
    }
}
