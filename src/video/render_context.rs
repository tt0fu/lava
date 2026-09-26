use std::sync::Arc;
use vulkano::{image::Image, pipeline::graphics::viewport::Viewport, swapchain::Swapchain};
use vulkano_taskgraph::{Id, graph::ExecutableTaskGraph};
use winit::window::Window;

use crate::{
    audio::stream::Stream,
    video::{buffers::Buffers, global_parameters::GlobalParameters},
};

const MAX_FRAMES_IN_FLIGHT: u32 = 2;
const MIN_SWAPCHAIN_IMAGES: u32 = MAX_FRAMES_IN_FLIGHT + 1;

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
    pub fn new() -> Self {
        todo!()
    }
}
