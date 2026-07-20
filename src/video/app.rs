use glam::vec2;
use std::{sync::Arc, time::Instant};
use vulkano::{
    VulkanError, VulkanLibrary,
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    image::ImageUsage,
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
    stats::frame_timer::FrameTimer,
    video::{render_task::RenderTask, shaders},
};

const MAX_FRAMES_IN_FLIGHT: u32 = 2;
const MIN_SWAPCHAIN_IMAGES: u32 = MAX_FRAMES_IN_FLIGHT + 1;

// pub enum BufferUpdateFrequency {
//     EveryFrame,
//     OnSwapchainRecreation,
//     Once,
// }

// pub trait BufferEmitter {}

// pub struct UpdateBuffer {
//     pub buffer: Id<Buffer>,
//     pub frequency: BufferUpdateFrequency,
// }

pub struct Buffers {
    pub global: Id<Buffer>,
    pub transforms: Vec<Id<Buffer>>,
}

pub struct RenderContext {
    pub window: Arc<Window>,
    pub swapchain_id: Id<Swapchain>,
    pub viewport: Viewport,
    pub recreate_swapchain: bool,
    pub task_graph: ExecutableTaskGraph<Self>,
    pub virtual_swapchain_id: Id<Swapchain>,

    pub start_time: Instant,
    pub buffers: Buffers,
}

pub struct App {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub resources: Arc<Resources>,
    pub flight_id: Id<Flight>,
    pub rcx: Option<RenderContext>,

    pub frame_timer: FrameTimer,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let library = unsafe { VulkanLibrary::new() }.unwrap();
        let required_extensions = Surface::required_extensions(event_loop);
        let instance = Instance::new(
            &library,
            &InstanceCreateInfo {
                // Enable enumerating devices that use non-conformant Vulkan implementations (e.g.,
                // MoltenVK).
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
        App {
            instance,
            device,
            queue,
            resources,
            flight_id,
            rcx,
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

        let buffer_create_info = BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        };
        let allocation_create_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        };
        let buffers = Buffers {
            global: self
                .resources
                .create_buffer(
                    &buffer_create_info,
                    &allocation_create_info,
                    DeviceLayout::new_sized::<shaders::GlobalParams>(),
                )
                .unwrap(),
        };

        task_graph.add_host_buffer_access(buffers.global, HostAccessType::Write);

        let rectangle_node_id = task_graph
            .create_task_node(
                "Rectangle",
                QueueFamilyType::Graphics,
                RenderTask::new(self, virtual_swapchain_id),
            )
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
        let rectangle_node = task_graph.task_node_mut(rectangle_node_id).unwrap();
        let subpass = rectangle_node.subpass().unwrap().clone();
        rectangle_node
            .task_mut()
            .downcast_mut::<RenderTask>()
            .unwrap()
            .create_data(
                self,
                &subpass,
                &buffers,
                vec2(window_size.width as f32, window_size.height as f32),
            );
        let recreate_swapchain = false;
        self.rcx = Some(RenderContext {
            window,
            swapchain_id,
            viewport,
            recreate_swapchain,
            task_graph,
            virtual_swapchain_id,
            buffers,
            start_time: Instant::now(),
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
                println!("TaskGraph execute call times:");
                self.frame_timer.print_results();
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                rcx.recreate_swapchain = true;
            }
            WindowEvent::RedrawRequested => {
                let window_size = rcx.window.inner_size();
                if window_size.width == 0 || window_size.height == 0 {
                    return;
                }
                if rcx.recreate_swapchain {
                    rcx.swapchain_id = self
                        .resources
                        .recreate_swapchain(rcx.swapchain_id, |create_info| SwapchainCreateInfo {
                            image_extent: window_size.into(),
                            ..*create_info
                        })
                        .expect("failed to recreate swapchain");
                    rcx.viewport.extent = window_size.into();
                    rcx.recreate_swapchain = false;
                }
                let flight = self.resources.flight(self.flight_id);
                flight.wait(None).unwrap();
                let resource_map =
                    resource_map!(&rcx.task_graph, rcx.virtual_swapchain_id => rcx.swapchain_id)
                        .unwrap();
                match {
                    self.frame_timer.start_frame();
                    let res = unsafe {
                        rcx.task_graph
                            .execute(resource_map, rcx, || rcx.window.pre_present_notify())
                    };
                    self.frame_timer.end_frame();
                    res
                } {
                    Ok(()) => {}
                    Err(ExecuteError::Swapchain {
                        error: VulkanError::OutOfDate,
                        ..
                    }) => {
                        rcx.recreate_swapchain = true;
                    }
                    Err(e) => {
                        panic!("failed to execute next frame: {e:?}");
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let rcx = self.rcx.as_mut().unwrap();
        rcx.window.request_redraw();
    }
}
