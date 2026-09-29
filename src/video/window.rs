use std::sync::Arc;

use glam::Vec2;
use vulkano::{
    device::Device,
    format::Format,
    image::ImageUsage,
    instance::Instance,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use vulkano_taskgraph::{Id, resource::Resources};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::Window};

use crate::video::MIN_SWAPCHAIN_IMAGES;

pub struct WindowState {
    pub window: Arc<Window>,
    pub swapchain_id: Id<Swapchain>,
    pub format: Format,
    pub recreate_requested: bool,
}

impl WindowState {
    pub fn new(
        event_loop: &ActiveEventLoop,
        instance: &Arc<Instance>,
        device: &Arc<Device>,
        resources: &Arc<Resources>,
    ) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let surface = Surface::from_window(instance, &window).unwrap();
        let (format, swapchain_id) =
            create_swapchain(device, resources, &surface, window.inner_size());
        Self {
            window,
            swapchain_id,
            format,
            recreate_requested: false,
        }
    }

    pub fn size(&self) -> Vec2 {
        let size = self.window.inner_size();
        Vec2::new(size.width as f32, size.height as f32)
    }

    pub fn recreate_swapchain(&mut self, resources: &Arc<Resources>) {
        let image_extent = self.window.inner_size().into();
        self.swapchain_id = resources
            .recreate_swapchain(self.swapchain_id, |create_info| SwapchainCreateInfo {
                image_extent,
                ..*create_info
            })
            .expect("failed to recreate swapchain");
        self.recreate_requested = false;
    }
}

fn create_swapchain(
    device: &Arc<Device>,
    resources: &Arc<Resources>,
    surface: &Arc<Surface>,
    window_size: PhysicalSize<u32>,
) -> (Format, Id<Swapchain>) {
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(surface, &Default::default())
        .unwrap();
    let (format, _) = device
        .physical_device()
        .surface_formats(surface, &Default::default())
        .unwrap()[0];
    let swapchain_id = resources
        .create_swapchain(
            surface,
            &SwapchainCreateInfo {
                min_image_count: surface_capabilities
                    .min_image_count
                    .max(MIN_SWAPCHAIN_IMAGES),
                image_format: format,
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
        .unwrap();
    (format, swapchain_id)
}
