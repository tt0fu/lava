use super::{
    super::audio::{Analyzer, Stream},
    Panel, PanelVariant, RenderEngine,
    shaders::{spectrogram, vs, waveform},
    window_size_dependent_setup,
};
use glam::{Vec2, f32::Mat3, vec2};
use std::{f32, sync::Arc};
use vulkano::{
    Validated, VulkanError,
    buffer::{BufferContents, allocator::SubbufferAllocator},
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
    pub stream: Stream<4096, 2>,
    pub analyzer: Analyzer<4096, 512, 48000>,
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
                        // .with_inner_size(winit::dpi::LogicalSize::new(1080, 1920)),
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

        let panels = vec![
            Panel {
                variant: PanelVariant::WAVEFORM,
                scale: vec2(1.0, 0.5),
                angle: 0.0,
                translation: vec2(0.0, 0.5),
            },
            Panel {
                variant: PanelVariant::WAVEFORM,
                scale: vec2(1.0, -0.5),
                angle: 0.0,
                translation: vec2(0.0, -0.5),
            },
            Panel {
                variant: PanelVariant::SPECTROGRAM,
                scale: vec2(1.0, -0.5),
                angle: 0.0,
                translation: vec2(0.0, 0.5),
            },
            Panel {
                variant: PanelVariant::SPECTROGRAM,
                scale: vec2(1.0, 0.5),
                angle: 0.0,
                translation: vec2(0.0, -0.5),
            },
            // Panel {
            //     variant: PanelVariant::WAVEFORM,
            //     scale: vec2(1920.0 / 1080.0, -0.5 * 1080.0 / 1920.0),
            //     angle: f32::consts::FRAC_PI_2,
            //     translation: vec2(0.5 * 1080.0 / 1920.0, 0.0),
            // },
            // Panel {
            //     variant: PanelVariant::WAVEFORM,
            //     scale: vec2(1920.0 / 1080.0, 0.5 * 1080.0 / 1920.0),
            //     angle: f32::consts::FRAC_PI_2,
            //     translation: vec2(-0.5 * 1080.0 / 1920.0, 0.0),
            // },
            // Panel {
            //     variant: PanelVariant::SPECTROGRAM,
            //     scale: vec2(1.0, -0.5),
            //     angle: 0.0,
            //     translation: vec2(0.0, 0.5),
            // },
            // Panel {
            //     variant: PanelVariant::SPECTROGRAM,
            //     scale: vec2(1.0, 0.5),
            //     angle: 0.0,
            //     translation: vec2(0.0, -0.5),
            // },
        ];

        let vs = vs::load(render_engine.device.clone())
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
            stream: Stream::new(48000, 2048),
            analyzer: Analyzer::new(),
        }
    }

    fn create_write_descriptor_set<T: BufferContents>(
        uniform_buffer_allocator: &SubbufferAllocator,
        binding: u32,
        content: T,
    ) -> WriteDescriptorSet {
        let buffer = uniform_buffer_allocator.allocate_sized().unwrap();
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

        let to_screen = Mat3::from_scale(Vec2::new(
            window_size.height as f32 / window_size.width as f32,
            1.0,
        ));
        let to_normalized = Mat3::from_scale(Vec2::new(
            window_size.width as f32 / window_size.height as f32,
            1.0,
        ));
        let new_samples = self.stream.get_samples();
        let mono = new_samples
            .iter()
            .map(|s| (s[0] + s[1]) / 2.0)
            .collect::<Vec<f32>>();
        for sample in &mono {
            self.analyzer.push(sample);
        }
        let samples_buffer = self.analyzer.get_buffer();
        let analysis_data = self.analyzer.get_analysis_data();

        for i in (0..(self.panels.len())).rev() {
            let panel = &self.panels[i];

            let layout = self.pipelines[i].layout().set_layouts()[0].clone();

            let transform_write = {
                let transform = to_screen
                    * Mat3::from_scale_angle_translation(
                        panel.scale,
                        panel.angle,
                        panel.translation,
                    )
                    * to_normalized;
                Self::create_write_descriptor_set(
                    &render_engine.uniform_buffer_allocator,
                    0,
                    vs::Transform {
                        transform: [
                            transform.x_axis.to_array().into(),
                            transform.y_axis.to_array().into(),
                            transform.z_axis.to_array().into(),
                        ],
                    },
                )
            };

            let writes = match panel.variant {
                PanelVariant::WAVEFORM => vec![
                    transform_write,
                    Self::create_write_descriptor_set(
                        &render_engine.uniform_buffer_allocator,
                        1,
                        waveform::ScaleX {
                            scale_x: ((window_size.width as f32 / window_size.height as f32)
                                * (panel.scale.x.abs() / panel.scale.y.abs()))
                            .into(),
                        },
                    ),
                    Self::create_write_descriptor_set(
                        &render_engine.uniform_buffer_allocator,
                        2,
                        waveform::Samples {
                            samples_start: (samples_buffer.start() as u32).into(),
                            samples_data: samples_buffer.data().map(|x| x.into()),
                        },
                    ),
                    Self::create_write_descriptor_set(
                        &render_engine.uniform_buffer_allocator,
                        3,
                        waveform::Stabilization {
                            period: analysis_data.period.into(),
                            focus: analysis_data.focus.into(),
                            center_sample: analysis_data.center_sample.into(),
                        },
                    ),
                    Self::create_write_descriptor_set(
                        &render_engine.uniform_buffer_allocator,
                        5,
                        waveform::Bass {
                            bass: analysis_data.bass.into(),
                            chrono: analysis_data.chrono.into(),
                        },
                    ),
                ],
                PanelVariant::SPECTROGRAM => vec![
                    transform_write,
                    Self::create_write_descriptor_set(
                        &render_engine.uniform_buffer_allocator,
                        4,
                        spectrogram::Dft {
                            dft: analysis_data.dft.map(|bin| [bin.x, bin.y].into()),
                        },
                    ),
                ],
            };

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
