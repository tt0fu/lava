use std::sync::Arc;

use vulkano::{
    VulkanLibrary,
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
    swapchain::Surface,
};
use winit::event_loop::EventLoop;

pub fn create_instance(event_loop: &EventLoop<()>, debug: bool) -> Arc<Instance> {
    let extensions = InstanceExtensions {
        ext_debug_utils: debug,
        ..InstanceExtensions::empty()
    };
    let library = unsafe { VulkanLibrary::new() }.unwrap();

    let required_extensions = Surface::required_extensions(event_loop);
    Instance::new(
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
    .unwrap()
}
