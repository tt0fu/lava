use std::sync::Arc;

use vulkano::{
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDeviceType,
    },
    instance::Instance,
};
use vulkano_taskgraph::descriptor_set::BindlessContext;
use winit::event_loop::EventLoop;

pub fn create_device(
    instance: &Arc<Instance>,
    event_loop: &EventLoop<()>,
) -> (Arc<Device>, Arc<Queue>) {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .into_iter()
        .filter(|physical_device| {
            physical_device
                .supported_extensions()
                .contains(&device_extensions)
        })
        .filter_map(|physical_device| {
            physical_device
                .queue_family_properties()
                .iter()
                .enumerate()
                .position(|(index, queue)| {
                    queue.queue_flags.intersects(QueueFlags::GRAPHICS)
                        && physical_device.presentation_support(index as u32, event_loop)
                })
                .map(|index| (physical_device, index as u32))
        })
        .min_by_key(|(physical_device, _)| device_rank(physical_device.properties().device_type))
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
                ..BindlessContext::required_features(instance)
            },
            queue_create_infos: &[QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();

    (device, queues[0].clone())
}

fn device_rank(device_type: PhysicalDeviceType) -> u32 {
    match device_type {
        PhysicalDeviceType::DiscreteGpu => 0,
        PhysicalDeviceType::IntegratedGpu => 1,
        PhysicalDeviceType::VirtualGpu => 2,
        PhysicalDeviceType::Cpu => 3,
        PhysicalDeviceType::Other => 4,
        _ => 5,
    }
}
