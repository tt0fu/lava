use crate::{
    audio::analyzer::AudioData,
    config::Config,
    video::{
        GlobalWrites, Mesh, Panel, Texture, shaders::load_vertex, window_size_dependent_setup,
    },
};

use glam::vec2;
use std::sync::Arc;
use vulkano::{
    Validated, VulkanError,
    buffer::allocator::SubbufferAllocator,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo,
        allocator::StandardCommandBufferAllocator,
    },
    descriptor_set::{DescriptorSet, allocator::StandardDescriptorSetAllocator},
    device::Device,
    device::Queue,
    format::Format,
    image::ImageUsage,
    instance::Instance,
    memory::allocator::StandardMemoryAllocator,
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, RenderPass},
    shader::EntryPoint,
    swapchain::{
        Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo, acquire_next_image,
    },
    sync::{self, GpuFuture},
};

use winit::{dpi::PhysicalSize, window::Window};

pub struct RenderContext {
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub vertex_shader: EntryPoint,
    pub fragment_shaders: Vec<EntryPoint>,
    pub panels: Vec<Panel>,
    pub pipelines: Vec<Arc<GraphicsPipeline>>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl RenderContext {
    pub fn new(
        instance: &Arc<Instance>,
        device: &Arc<Device>,
        memory_allocator: &Arc<StandardMemoryAllocator>,
        window: &Arc<Box<dyn Window>>,
        config: &Config,
    ) -> Self {
        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();
        let window_size = window.surface_size();

        let (swapchain, images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();
            let (image_format, _) = device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0];

            Swapchain::new(
                device.clone(),
                surface,
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
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

        let render_pass = vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth_stencil: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil},
            },
        )
        .unwrap();

        let panels = config.panels.clone();

        let vertex_shader = load_vertex(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let fragment_shaders = panels
            .iter()
            .map(|p| p.get_shader_entry_point(&device, &config))
            .collect();

        let (framebuffers, pipelines) = window_size_dependent_setup(
            window_size,
            &images,
            &render_pass,
            &memory_allocator,
            &vertex_shader,
            &fragment_shaders,
        );

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        Self {
            swapchain,
            render_pass,
            framebuffers,
            vertex_shader,
            fragment_shaders,
            panels,
            pipelines,
            recreate_swapchain: false,
            previous_frame_end,
        }
    }

    pub fn redraw(
        &mut self,
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        memory_allocator: &Arc<StandardMemoryAllocator>,
        descriptor_set_allocator: &Arc<StandardDescriptorSetAllocator>,
        command_buffer_allocator: &Arc<StandardCommandBufferAllocator>,
        uniform_buffer_allocator: &SubbufferAllocator,
        storage_buffer_allocator: &SubbufferAllocator,
        mesh: &Mesh,
        texture: &Option<Texture>,
        window_size: &PhysicalSize<u32>,
        audio_data: &AudioData,
    ) {
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            let (new_swapchain, new_images) = self
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: window_size.clone().into(),
                    ..self.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            self.swapchain = new_swapchain;
            (self.framebuffers, self.pipelines) = window_size_dependent_setup(
                window_size.clone(),
                &new_images,
                &self.render_pass,
                &memory_allocator,
                &self.vertex_shader,
                &self.fragment_shaders,
            );
            self.recreate_swapchain = false;
        }

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.0, 0.0, 0.0, 1.0].into()), // background color
                        Some(1f32.into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                Default::default(),
            )
            .unwrap()
            .bind_vertex_buffers(0, (mesh.vertex_buffer.clone(), mesh.uvs_buffer.clone()))
            .unwrap()
            .bind_index_buffer(mesh.index_buffer.clone())
            .unwrap();

        let global_writes = GlobalWrites::new(
            &uniform_buffer_allocator,
            &storage_buffer_allocator,
            &texture,
            &audio_data,
        );

        for i in 0..(self.panels.len()) {
            let panel = &self.panels[i];

            let layout = self.pipelines[i].layout().set_layouts()[0].clone();

            let writes = panel.get_write_descriptor_sets(
                &uniform_buffer_allocator,
                vec2(window_size.width as f32, window_size.height as f32),
                global_writes.clone(),
            );

            let descriptor_set =
                DescriptorSet::new(descriptor_set_allocator.clone(), layout, writes, []).unwrap();

            builder
                .bind_pipeline_graphics(self.pipelines[i].clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.pipelines[i].layout().clone(),
                    0,
                    descriptor_set,
                )
                .unwrap();
            unsafe { builder.draw_indexed(mesh.index_buffer.len() as u32, 1, 0, 0, 0) }.unwrap();
        }

        builder.end_render_pass(Default::default()).unwrap();

        let command_buffer = builder.build().unwrap();
        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                self.previous_frame_end = Some(sync::now(device.clone()).boxed());
            }
        }
    }
}
