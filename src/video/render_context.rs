use super::{RenderEngine, fs, vs, window_size_dependent_setup};
use crate::audio::{Analyzer, Stream};
use std::sync::Arc;
use vulkano::{
    Validated, VulkanError,
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
use winit::{event_loop::ActiveEventLoop, platform::x11::WindowAttributesExtX11, window::Window};

pub struct RenderContext {
    pub window: Arc<Window>,
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub vs: EntryPoint,
    pub fs: EntryPoint,
    pub pipeline: Arc<GraphicsPipeline>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub stream: Stream<4096, 2>,
    pub analyzer: Analyzer<4096, 512, 48000>,
}

impl RenderContext {
    pub fn new(render_engine: &RenderEngine, event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("lava visualizer")
                        .with_name("org.ttofu.lava", "lava window instance")
                        .with_decorations(false)
                        .with_resizable(false)
                        .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080)),
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

        let vs = vs::load(render_engine.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(render_engine.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let (framebuffers, pipeline) = window_size_dependent_setup(
            window_size,
            &images,
            &render_pass,
            &render_engine.memory_allocator,
            &vs,
            &fs,
        );

        let previous_frame_end = Some(sync::now(render_engine.device.clone()).boxed());

        // let time_start = Instant::now();

        Self {
            window,
            swapchain,
            render_pass,
            framebuffers,
            vs,
            fs,
            pipeline,
            recreate_swapchain: false,
            previous_frame_end,
            // time_start,
            stream: Stream::new(48000, 4096),
            analyzer: Analyzer::new(),
        }
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
            (self.framebuffers, self.pipeline) = window_size_dependent_setup(
                window_size,
                &new_images,
                &self.render_pass,
                &render_engine.memory_allocator,
                &self.vs,
                &self.fs,
            );
            self.recreate_swapchain = false;
        }

        let uniform_buffer = {
            let new_samples = self.stream.get_samples();
            let mono = new_samples
                .iter()
                .map(|s| (s[0] + s[1]) / 2.0)
                .collect::<Vec<f32>>();
            for sample in &mono {
                self.analyzer.push(sample);
            }

            let samples_buffer = self.analyzer.get_buffer();
            let analysis_data = self.analyzer.get_analysis_info();

            let uniform_data = fs::Data {
                scale_x: (window_size.width as f32 / window_size.height as f32).into(),
                samples_start: (samples_buffer.start() as u32).into(),
                samples_data: samples_buffer.data().map(|x| x.into()),
                period: analysis_data.period.into(),
                focus: analysis_data.focus.into(),
                center_sample: analysis_data.center_sample.into(),
                bass: analysis_data.bass.into(),
                chrono: analysis_data.chrono.into(),
                line_width: 50.0,
            };

            let buffer = render_engine
                .uniform_buffer_allocator
                .allocate_sized()
                .unwrap();
            *buffer.write().unwrap() = uniform_data;

            buffer
        };

        let layout = &self.pipeline.layout().set_layouts()[0];
        let descriptor_set = DescriptorSet::new(
            render_engine.descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
            [],
        )
        .unwrap();

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
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
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
        unsafe { builder.draw_indexed(render_engine.mesh.index_buffer.len() as u32, 1, 0, 0, 0) }
            .unwrap();

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
