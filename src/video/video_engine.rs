use crate::{
    audio::AudioData,
    config::Config,
    video::{Mesh, RenderContext, Texture},
};

use std::sync::Arc;
use vulkano::{
    VulkanLibrary,
    buffer::{
        BufferUsage,
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
    },
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::PhysicalDeviceType,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator},
    swapchain::Surface,
};
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::Window};

pub struct VideoEngine {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub uniform_buffer_allocator: SubbufferAllocator,
    pub storage_buffer_allocator: SubbufferAllocator,
    pub mesh: Mesh,
    pub texture: Texture,

    pub context: Option<RenderContext>,
}

impl VideoEngine {
    pub fn new(event_loop: &EventLoop) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let required_extensions = Surface::required_extensions(event_loop).unwrap();
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
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
                            && p.presentation_support(i as u32, event_loop).unwrap()
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
            .unwrap();

        println!(
            "Using video device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let uniform_buffer_allocator = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let storage_buffer_allocator = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::STORAGE_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let mesh = Mesh::new(&memory_allocator);

        let texture = Texture::new(
            &device,
            &queue,
            &memory_allocator,
            &command_buffer_allocator,
        );

        Self {
            instance,
            device,
            queue,
            memory_allocator,
            descriptor_set_allocator,
            command_buffer_allocator,
            uniform_buffer_allocator,
            storage_buffer_allocator,
            mesh,
            texture,
            context: None,
        }
    }

    pub fn init(&mut self, window: &Arc<Box<dyn Window>>, config: &Config) {
        self.context = Some(RenderContext::new(
            &self.instance,
            &self.device,
            &self.memory_allocator,
            &window,
            &config,
        ));
    }

    pub fn resize(&mut self) {
        self.context.as_mut().unwrap().recreate_swapchain = true;
    }

    pub fn redraw(&mut self, window_size: &PhysicalSize<u32>, audio_data: &AudioData) {
        self.context.as_mut().unwrap().redraw(
            &self.device,
            &self.queue,
            &self.memory_allocator,
            &self.descriptor_set_allocator,
            &self.command_buffer_allocator,
            &self.uniform_buffer_allocator,
            &self.storage_buffer_allocator,
            &self.mesh,
            &self.texture,
            &window_size,
            &audio_data,
        );
    }
}
