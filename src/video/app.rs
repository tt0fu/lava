use std::sync::Arc;
use vulkano::{
    VulkanLibrary,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    instance::{
        Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions,
        debug::{
            DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
            DebugUtilsMessengerCallback, DebugUtilsMessengerCreateInfo,
        },
    },
    swapchain::Surface,
};
use vulkano_taskgraph::{
    Id,
    descriptor_set::BindlessContext,
    resource::{Flight, Resources, ResourcesCreateInfo},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::{
    audio::{audio_settings::AudioSettings, stream::Stream},
    stats::frame_timer::FrameTimer,
    video::{render_context::RenderContext, scene_data::SceneData},
};

pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;
pub const MIN_SWAPCHAIN_IMAGES: u32 = MAX_FRAMES_IN_FLIGHT + 1;

pub struct App {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub resources: Arc<Resources>,
    pub flight_id: Id<Flight>,

    pub scene_data: Arc<SceneData>,
    pub audio_settings: Arc<AudioSettings>,

    pub render_context: Option<RenderContext>,

    pub stream: Arc<Stream>,

    pub frame_timer: FrameTimer,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>, audio_settings: AudioSettings, debug: bool) -> Self {
        let extensions = InstanceExtensions {
            ext_debug_utils: debug,
            ..InstanceExtensions::empty()
        };
        let library = unsafe { VulkanLibrary::new() }.unwrap();

        let required_extensions = Surface::required_extensions(event_loop);
        let instance = Instance::new(
            &library,
            &InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_layers: if debug {
                    &["VK_LAYER_KHRONOS_validation"]
                } else {
                    &[]
                },
                enabled_extensions: &required_extensions.union(&extensions),
                ..Default::default()
            },
        )
        .unwrap();

        if debug {
            let _debug_callback = unsafe {
                DebugUtilsMessenger::new(
                    &instance,
                    &DebugUtilsMessengerCreateInfo {
                        message_severity: DebugUtilsMessageSeverity::ERROR
                            | DebugUtilsMessageSeverity::WARNING
                            | DebugUtilsMessageSeverity::INFO
                            | DebugUtilsMessageSeverity::VERBOSE,
                        message_type: DebugUtilsMessageType::GENERAL
                            | DebugUtilsMessageType::VALIDATION
                            | DebugUtilsMessageType::PERFORMANCE,
                        ..DebugUtilsMessengerCreateInfo::new(&DebugUtilsMessengerCallback::new(
                            |message_severity, message_type, callback_data| {
                                let severity = if message_severity
                                    .intersects(DebugUtilsMessageSeverity::ERROR)
                                {
                                    "error"
                                } else if message_severity
                                    .intersects(DebugUtilsMessageSeverity::WARNING)
                                {
                                    "warning"
                                } else if message_severity
                                    .intersects(DebugUtilsMessageSeverity::INFO)
                                {
                                    "information"
                                } else if message_severity
                                    .intersects(DebugUtilsMessageSeverity::VERBOSE)
                                {
                                    "verbose"
                                } else {
                                    panic!("no-impl");
                                };

                                let ty = if message_type.intersects(DebugUtilsMessageType::GENERAL)
                                {
                                    "general"
                                } else if message_type.intersects(DebugUtilsMessageType::VALIDATION)
                                {
                                    "validation"
                                } else if message_type
                                    .intersects(DebugUtilsMessageType::PERFORMANCE)
                                {
                                    "performance"
                                } else {
                                    panic!("no-impl");
                                };

                                println!(
                                    "{} {} {}: {}",
                                    callback_data.message_id_name.unwrap_or("unknown"),
                                    ty,
                                    severity,
                                    callback_data.message
                                );
                            },
                        ))
                    },
                )
            }
            .ok();
        }

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .into_iter()
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
        let (device, queues) = Device::new(
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

        let scene_data = SceneData::new(&device);

        let queue = queues[0].clone();
        let resources = Resources::new(
            &device,
            &ResourcesCreateInfo {
                bindless_context: Some(&Default::default()),
                ..Default::default()
            },
        )
        .unwrap();
        let flight_id = resources.create_flight(MAX_FRAMES_IN_FLIGHT).unwrap();
        let render_context = None;
        let stream = Arc::new(Stream::new(&audio_settings).unwrap());
        App {
            instance,
            device,
            queue,
            resources,
            flight_id,
            scene_data: Arc::new(scene_data),
            audio_settings: Arc::new(audio_settings),
            render_context,
            stream,
            frame_timer: FrameTimer::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.render_context = Some(RenderContext::new(
            &event_loop,
            &self.instance,
            &self.device,
            &self.queue,
            &self.resources,
            self.flight_id,
            &self.scene_data,
            &self.audio_settings,
            &self.stream,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let rcx = self.render_context.as_mut().unwrap();
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
                rcx.redraw(&self.resources, self.flight_id);
                self.frame_timer.end_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let rcx = self.render_context.as_mut().unwrap();
        rcx.window.request_redraw();
    }
}
