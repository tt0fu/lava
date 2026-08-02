use crate::video::{
    app::{App, Buffers, RenderContext},
    audio_settings::AudioSettings,
    model::{MyVertex, VERTICES},
    parameters::Write,
    scene_data::SceneData,
    shaders,
};
use std::{slice, sync::Arc};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::Device,
    image::Image,
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
    pipeline::{
        ComputePipeline, DynamicState, GraphicsPipeline, PipelineLayout,
        PipelineShaderStageCreateInfo,
        compute::ComputePipelineCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology::TriangleStrip},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition, VertexInputState},
            viewport::ViewportState,
        },
    },
    render_pass::Subpass,
    shader::EntryPoint,
    swapchain::Swapchain,
    sync::{AccessFlags, PipelineStages},
};
use vulkano_taskgraph::{
    ClearValues, Id, Task, TaskContext,
    command_buffer::{DependencyInfo, MemoryBarrier, RecordingCommandBuffer},
    descriptor_set::{BindlessContext, StorageBufferId},
    resource::HostAccessType,
};

// Information needed to draw all panels with a specific shader
pub struct PipelineData {
    pub pipeline: Arc<GraphicsPipeline>,
    pub panels: Vec<shaders::PushConstants>,
}

// Informaton needed to draw all panels
pub struct RenderData {
    pub layout: Arc<PipelineLayout>,
    pub pipelines: Vec<PipelineData>,
    pub dft_pipeline: Arc<ComputePipeline>,
    pub analysis_pipeline: Arc<ComputePipeline>,
    pub compute_push_constants: shaders::ComputePushConstants,
}

pub struct RenderTask {
    pub vertex_buffer_id: Id<Buffer>,
    pub swapchain_id: Id<Swapchain>,
    pub depth_buffer_id: Id<Image>,
    pub scene_data: Arc<SceneData>,
    pub audio_settings: Arc<AudioSettings>,
    pub render_data: Option<RenderData>,
}

fn create_graphics_pipeline(
    device: &Arc<Device>,
    subpass: &Subpass,
    vertex_input_state: &VertexInputState,
    layout: &Arc<PipelineLayout>,
    stages: &[PipelineShaderStageCreateInfo],
) -> Arc<GraphicsPipeline> {
    GraphicsPipeline::new(
        &device,
        None,
        &GraphicsPipelineCreateInfo {
            stages: &stages,
            vertex_input_state: Some(&vertex_input_state),
            input_assembly_state: Some(&InputAssemblyState {
                topology: TriangleStrip,
                ..Default::default()
            }),
            viewport_state: Some(&ViewportState::default()),
            rasterization_state: Some(&RasterizationState::default()),
            depth_stencil_state: Some(&DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(&MultisampleState::default()),
            color_blend_state: Some(&ColorBlendState {
                attachments: &[ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend::alpha()),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            dynamic_state: &[DynamicState::Viewport],
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::new(&layout.clone())
        },
    )
    .unwrap()
}

fn create_compute_pipeline(
    device: &Arc<Device>,
    bcx: &BindlessContext,
    entry_point: &EntryPoint,
) -> Arc<ComputePipeline> {
    let stage = PipelineShaderStageCreateInfo::new(&entry_point);
    let layout = bcx
        .pipeline_layout_from_stages(slice::from_ref(&stage))
        .unwrap();
    ComputePipeline::new(
        &device,
        None,
        &ComputePipelineCreateInfo::new(stage, &layout),
    )
    .unwrap()
}

impl RenderTask {
    pub fn new(app: &mut App, swapchain_id: Id<Swapchain>, depth_buffer_id: Id<Image>) -> Self {
        let vertex_buffer_id = app
            .resources
            .create_buffer(
                &BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                &AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                DeviceLayout::for_value(VERTICES.as_slice()).unwrap(),
            )
            .unwrap();

        unsafe {
            vulkano_taskgraph::execute(
                &app.queue,
                &app.resources,
                app.flight_id,
                |_cbf, tcx| {
                    tcx.try_write_buffer::<[MyVertex]>(vertex_buffer_id, ..)?
                        .copy_from_slice(&VERTICES);

                    Ok(())
                },
                [(vertex_buffer_id, HostAccessType::Write)],
                [],
                [],
            )
        }
        .unwrap();

        let render_data = None;

        Self {
            vertex_buffer_id,
            swapchain_id,
            depth_buffer_id,
            scene_data: app.scene_data.clone(),
            audio_settings: app.audio_settings.clone(),
            render_data,
        }
    }

    pub fn create_render_data(&mut self, app: &App, subpass: &Subpass, buffers: &Buffers) {
        let vertex_shader = unsafe { shaders::load_vertex(&app.device) }
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = [MyVertex::per_vertex()].definition(&vertex_shader).unwrap();

        let all_stages = std::iter::once(PipelineShaderStageCreateInfo::new(&vertex_shader))
            .chain(
                app.scene_data
                    .shaders
                    .iter()
                    .map(|e| PipelineShaderStageCreateInfo::new(&e)),
            )
            .collect::<Vec<PipelineShaderStageCreateInfo>>();

        let bcx = app.resources.bindless_context().unwrap();

        let layout = bcx
            .pipeline_layout_from_stages(all_stages.as_slice())
            .unwrap();

        let dft_pipeline = create_compute_pipeline(
            &app.device,
            bcx,
            &unsafe { shaders::load_dft(&app.device) }
                .unwrap()
                .specialize(&[(0, self.audio_settings.sample_rate.into())])
                .entry_point("main")
                .unwrap(),
        );
        let analysis_pipeline = create_compute_pipeline(
            &app.device,
            bcx,
            &unsafe { shaders::load_analysis(&app.device) }
                .unwrap()
                .specialize(&[(0, self.audio_settings.sample_rate.into())])
                .entry_point("main")
                .unwrap(),
        );

        let global_buffer_id = bcx
            .global_set()
            .create_storage_buffer(buffers.global, 0, None)
            .unwrap();
        let waveform_buffer_id = bcx
            .global_set()
            .create_storage_buffer(buffers.waveform, 0, None)
            .unwrap();
        let dft_buffer_id = bcx
            .global_set()
            .create_storage_buffer(buffers.dft, 0, None)
            .unwrap();
        let bands_buffer_id = bcx
            .global_set()
            .create_storage_buffer(buffers.bands, 0, None)
            .unwrap();
        let transform_buffer_ids = buffers
            .transforms
            .iter()
            .map(|&id| bcx.global_set().create_storage_buffer(id, 0, None).unwrap())
            .collect::<Vec<StorageBufferId>>();
        let material_buffer_ids = buffers
            .materials
            .iter()
            .map(|&id| bcx.global_set().create_storage_buffer(id, 0, None).unwrap())
            .collect::<Vec<StorageBufferId>>();

        let min_order = app.scene_data.panels.iter().map(|p| p.order).min().unwrap();
        let max_order = app.scene_data.panels.iter().map(|p| p.order).max().unwrap();

        let pushes = app
            .scene_data
            .panels
            .iter()
            .map(|p| shaders::PushConstants {
                global_buffer_id,
                waveform_buffer_id,
                dft_buffer_id,
                bands_buffer_id,

                transform_buffer_id: transform_buffer_ids[p.transform_id],
                material_buffer_id: material_buffer_ids[p.material_id],

                panel_depth: (max_order - p.order) as f32 / (max_order + 1 - min_order) as f32,
            })
            .collect::<Vec<shaders::PushConstants>>();

        let mut panels_per_pipeline = vec![vec![]; app.scene_data.shaders.len()];

        app.scene_data
            .panels
            .iter()
            .enumerate()
            .for_each(|(ind, p)| {
                panels_per_pipeline[app.scene_data.materials[p.material_id].shader_id].push(ind)
            });

        let pipelines = app
            .scene_data
            .shaders
            .iter()
            .enumerate()
            .map(|(ind, e)| PipelineData {
                pipeline: create_graphics_pipeline(
                    &app.device,
                    &subpass,
                    &vertex_input_state,
                    &layout.clone(),
                    &[
                        PipelineShaderStageCreateInfo::new(&vertex_shader),
                        PipelineShaderStageCreateInfo::new(&e),
                    ],
                ),
                panels: panels_per_pipeline[ind]
                    .iter()
                    .map(|&i| pushes[i])
                    .collect(),
            })
            .collect();

        self.render_data = Some(RenderData {
            layout: layout.clone(),
            pipelines,
            dft_pipeline,
            analysis_pipeline,
            compute_push_constants: shaders::ComputePushConstants {
                global_buffer_id,
                waveform_buffer_id,
                dft_buffer_id,
                bands_buffer_id,
            },
        });
    }
}

impl Task for RenderTask {
    type World = RenderContext;

    fn clear_values(&self, clear_values: &mut ClearValues<'_>, _world: &Self::World) {
        let bg: [f32; 3] = self.scene_data.background_color.into();
        clear_values.set(self.swapchain_id.current_image_id(), bg);
        clear_values.set(self.depth_buffer_id, [1.0]);
    }

    unsafe fn execute(
        &self,
        cbf: &mut RecordingCommandBuffer<'_>,
        tcx: &mut TaskContext<'_>,
        rcx: &Self::World,
    ) -> vulkano_taskgraph::TaskResult {
        unsafe {
            let pass_data = self.render_data.as_ref().unwrap();

            rcx.global_parameters.write(rcx.buffers.global, tcx);
            rcx.stream.write(rcx.buffers.waveform, tcx);

            cbf.push_constants(
                pass_data.dft_pipeline.layout(),
                0,
                &pass_data.compute_push_constants,
            );

            cbf.bind_pipeline(&pass_data.dft_pipeline);
            cbf.dispatch([
                (self.audio_settings.dft_bin_count as u32).div_ceil(64),
                1,
                1,
            ]);
            cbf.pipeline_barrier(&DependencyInfo {
                memory_barriers: &[MemoryBarrier {
                    src_stages: PipelineStages::COMPUTE_SHADER,
                    src_access: AccessFlags::SHADER_WRITE,
                    dst_stages: PipelineStages::COMPUTE_SHADER,
                    dst_access: AccessFlags::SHADER_READ,
                    ..Default::default()
                }],
                ..Default::default()
            });

            cbf.bind_pipeline(&pass_data.analysis_pipeline);
            cbf.dispatch([1, 1, 1]);
            cbf.pipeline_barrier(&DependencyInfo {
                memory_barriers: &[MemoryBarrier {
                    src_stages: PipelineStages::COMPUTE_SHADER,
                    src_access: AccessFlags::SHADER_WRITE,
                    dst_stages: PipelineStages::FRAGMENT_SHADER,
                    dst_access: AccessFlags::SHADER_READ,
                    ..Default::default()
                }],
                ..Default::default()
            });

            cbf.set_viewport(0, slice::from_ref(&rcx.viewport));
            cbf.bind_vertex_buffers(0, &[self.vertex_buffer_id], &[0], &[], &[]);
            if rcx.rewrite_transforms {
                for i in 0..self.scene_data.transforms.len() {
                    self.scene_data.transforms[i].write(
                        rcx.viewport.extent.into(),
                        rcx.buffers.transforms[i],
                        tcx,
                    );
                }
            }

            for pipeline_data in &pass_data.pipelines {
                cbf.bind_pipeline(&pipeline_data.pipeline);
                for push_constant in &pipeline_data.panels {
                    cbf.push_constants(&pass_data.layout, 0, push_constant);
                    cbf.draw(VERTICES.len() as u32, 1, 0, 0);
                }
            }
        };
        Ok(())
    }
}
