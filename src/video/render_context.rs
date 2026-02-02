use super::{
    super::audio::{Analyzer, Stream},
    BIN_COUNT, PANELS, Panel, RenderEngine, SAMPLE_COUNT, SAMPLE_RATE, WINDOW_SIZE, shaders,
    shaders::{Bass, Dft, Samples, Stabilization},
    window_size_dependent_setup,
};
use glam::vec2;
use std::{array, f32, sync::Arc};
use vulkano::{
    Validated, VulkanError,
    buffer::{BufferContents, Subbuffer, allocator::SubbufferAllocator},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo},
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    format::Format,
    image::ImageUsage,
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, RenderPass},
    shader::EntryPoint,
    swapchain::{
        Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo, acquire_next_image,
    },
    sync::{self, GpuFuture},
};

#[cfg(target_os = "linux")]
use winit::platform::wayland::WindowAttributesExtWayland;
use winit::{event_loop::ActiveEventLoop, window::Window, window::WindowAttributes};

#[derive(Clone)]
pub struct GlobalWrites {
    pub samples: WriteDescriptorSet,
    pub stabilization: WriteDescriptorSet,
    pub dft: WriteDescriptorSet,
    pub bass: WriteDescriptorSet,
    pub image_sampler: WriteDescriptorSet,
    pub image_view: WriteDescriptorSet,
}

pub struct RenderContext {
    pub window: Arc<Window>,
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub vs: EntryPoint,
    pub fs: Vec<EntryPoint>,
    pub panels: Vec<Panel>,
    pub pipelines: Vec<Arc<GraphicsPipeline>>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub stream: Stream,
    pub analyzer: Analyzer,
}

impl RenderContext {
    #[cfg(not(target_os = "linux"))]
    fn spawn_window() -> WindowAttributes {
        Window::default_attributes()
    }

    #[cfg(target_os = "linux")]
    fn spawn_window() -> WindowAttributes {
        Window::default_attributes().with_name("org.ttofu.lava", "lava window instance")
    }

    pub fn new(render_engine: &RenderEngine, event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(
                    Self::spawn_window()
                        .with_title("lava visualizer")
                        .with_decorations(false)
                        .with_resizable(false)
                        .with_inner_size(WINDOW_SIZE),
                )
                .unwrap(),
        );
        let surface = Surface::from_window(render_engine.instance.clone(), window.clone()).unwrap();
        let window_size = window.inner_size();

        let (swapchain, images) = {
            let surface_capabilities = render_engine
                .device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();
            let (image_format, _) = render_engine
                .device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0];

            Swapchain::new(
                render_engine.device.clone(),
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

        let render_pass = vulkano::single_pass_renderpass!(render_engine.device.clone(),
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

        let panels = Vec::from(PANELS);

        let vs = shaders::load_vertex(render_engine.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let fs = panels
            .iter()
            .map(|p| p.get_shader_entry_point(&render_engine.device))
            .collect();

        let (framebuffers, pipelines) = window_size_dependent_setup(
            window_size,
            &images,
            &render_pass,
            &render_engine.memory_allocator,
            &vs,
            &fs,
        );

        let previous_frame_end = Some(sync::now(render_engine.device.clone()).boxed());

        Self {
            window,
            swapchain,
            render_pass,
            framebuffers,
            vs,
            fs,
            panels,
            pipelines,
            recreate_swapchain: false,
            previous_frame_end,
            stream: Stream::new(SAMPLE_RATE, 2048, 2048),
            analyzer: Analyzer::new(SAMPLE_COUNT, BIN_COUNT, SAMPLE_RATE),
        }
    }

    fn create_write_descriptor_set<T: BufferContents>(
        buffer_allocator: &SubbufferAllocator,
        binding: u32,
        content: T,
    ) -> WriteDescriptorSet {
        let buffer = buffer_allocator.allocate_sized().unwrap();
        *buffer.write().unwrap() = content;
        WriteDescriptorSet::buffer(binding, buffer)
    }

    pub fn redraw(&mut self, render_engine: &RenderEngine) {
        let window_size = self.window.inner_size();

        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            let (new_swapchain, new_images) = self
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: window_size.into(),
                    ..self.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            self.swapchain = new_swapchain;
            (self.framebuffers, self.pipelines) = window_size_dependent_setup(
                window_size,
                &new_images,
                &self.render_pass,
                &render_engine.memory_allocator,
                &self.vs,
                &self.fs,
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
            render_engine.command_buffer_allocator.clone(),
            render_engine.queue.queue_family_index(),
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
            .bind_vertex_buffers(
                0,
                (
                    render_engine.mesh.vertex_buffer.clone(),
                    render_engine.mesh.uvs_buffer.clone(),
                ),
            )
            .unwrap()
            .bind_index_buffer(render_engine.mesh.index_buffer.clone())
            .unwrap();

        self.analyzer.update(&mut self.stream);

        let screen_scale = vec2(window_size.width as f32, window_size.height as f32);
        let analysis_data = self.analyzer.analyze();
        let samples_buffer = self.analyzer.get_buffer();

        let samples_start = (samples_buffer.start() as u32).into();
        let samples_data: [_; SAMPLE_COUNT] = array::from_fn(|i| samples_buffer.data()[i].into());

        let period = analysis_data.period.into();
        let focus = analysis_data.focus.into();
        let center_sample = analysis_data.center_sample.into();

        let dft: [_; BIN_COUNT] = array::from_fn(|i| {
            let bin = analysis_data.dft[i];
            [bin.x, bin.y].into()
        });

        let bass = analysis_data.bass.into();
        let chrono = analysis_data.chrono.into();

        let global_writes = GlobalWrites {
            samples: {
                let buffer: Subbuffer<Samples> = render_engine
                    .storage_buffer_allocator
                    .allocate_unsized(SAMPLE_COUNT as u64)
                    .unwrap();
                let mut guard = buffer.write().unwrap();
                guard.samples_start = samples_start;
                guard.samples_data.copy_from_slice(&samples_data);
                drop(guard);
                WriteDescriptorSet::buffer(2, buffer)
            },
            stabilization: Self::create_write_descriptor_set(
                &render_engine.uniform_buffer_allocator,
                3,
                Stabilization {
                    period,
                    focus,
                    center_sample,
                },
            ),
            dft: {
                let buffer: Subbuffer<Dft> = render_engine
                    .storage_buffer_allocator
                    .allocate_unsized(BIN_COUNT as u64)
                    .unwrap();
                let mut guard = buffer.write().unwrap();
                guard.dft.copy_from_slice(&dft);
                drop(guard);
                WriteDescriptorSet::buffer(4, buffer)
            },
            bass: Self::create_write_descriptor_set(
                &render_engine.uniform_buffer_allocator,
                5,
                Bass { bass, chrono },
            ),
            image_sampler: WriteDescriptorSet::sampler(6, render_engine.sampler.clone()),
            image_view: WriteDescriptorSet::image_view(7, render_engine.texture.clone()),
        };

        for i in 0..(self.panels.len()) {
            let panel = &self.panels[i];

            let layout = self.pipelines[i].layout().set_layouts()[0].clone();

            let writes = panel.get_write_descriptor_sets(
                &render_engine.uniform_buffer_allocator,
                screen_scale,
                global_writes.clone(),
            );

            let descriptor_set = DescriptorSet::new(
                render_engine.descriptor_set_allocator.clone(),
                layout,
                writes,
                [],
            )
            .unwrap();

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
            unsafe {
                builder.draw_indexed(render_engine.mesh.index_buffer.len() as u32, 1, 0, 0, 0)
            }
            .unwrap();
        }

        builder.end_render_pass(Default::default()).unwrap();

        let command_buffer = builder.build().unwrap();
        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(render_engine.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                render_engine.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(render_engine.device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                self.previous_frame_end = Some(sync::now(render_engine.device.clone()).boxed());
            }
        }
    }
}
