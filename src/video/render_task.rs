use crate::video::{
    app::{App, Buffers, RenderContext},
    model::{MyVertex, VERTICES},
    panel::transform::{AnchorType, Transform, Unit, Vector},
    shaders,
};
use glam::{Mat3, Vec2, vec2};
use std::{slice, sync::Arc, time::Instant};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::Device,
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            input_assembly::{InputAssemblyState, PrimitiveTopology::TriangleStrip},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition, VertexInputState},
            viewport::ViewportState,
        },
    },
    render_pass::Subpass,
    swapchain::Swapchain,
};
use vulkano_taskgraph::{
    ClearValues, Id, Task, TaskContext, command_buffer::RecordingCommandBuffer,
    descriptor_set::StorageBufferId, resource::HostAccessType,
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
}

pub struct RenderTask {
    pub vertex_buffer_id: Id<Buffer>,
    pub swapchain_id: Id<Swapchain>,
    pub pass_data: Option<RenderData>,
}

fn create_pipeline(
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

impl RenderTask {
    pub fn new(app: &mut App, swapchain_id: Id<Swapchain>) -> Self {
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

        let pass_data = None;

        Self {
            pass_data,
            vertex_buffer_id,
            swapchain_id,
        }
    }

    pub fn create_data(
        &mut self,
        app: &App,
        subpass: &Subpass,
        buffers: &Buffers,
        resolution: Vec2,
    ) {
        let vertex_shader = unsafe { shaders::load_vertex(&app.device) }
            .unwrap()
            .entry_point("main")
            .unwrap();

        let fragment_shader = unsafe { shaders::load_fragment(&app.device) }
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = [MyVertex::per_vertex()].definition(&vertex_shader).unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(&vertex_shader),
            PipelineShaderStageCreateInfo::new(&fragment_shader),
        ];

        let bcx = app.resources.bindless_context().unwrap();

        let layout = bcx.pipeline_layout_from_stages(&stages).unwrap();

        let buffer_create_info = BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        };
        let allocation_create_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        };

        let transform_buffer_ids = vec![
            app.resources
                .create_buffer(
                    &buffer_create_info,
                    &allocation_create_info,
                    DeviceLayout::new_sized::<shaders::Transform>(),
                )
                .unwrap(),
            app.resources
                .create_buffer(
                    &buffer_create_info,
                    &allocation_create_info,
                    DeviceLayout::new_sized::<shaders::Transform>(),
                )
                .unwrap(),
        ];
        let local_buffer_ids = vec![
            app.resources
                .create_buffer(
                    &buffer_create_info,
                    &allocation_create_info,
                    DeviceLayout::new_sized::<shaders::LocalParams>(),
                )
                .unwrap(),
            app.resources
                .create_buffer(
                    &buffer_create_info,
                    &allocation_create_info,
                    DeviceLayout::new_sized::<shaders::LocalParams>(),
                )
                .unwrap(),
        ];
        let transform0 = Transform {
            anchor_type: AnchorType::TopLeft,
            anchor: Vector {
                value: vec2(0.0, 0.0),
                unit: Unit::Screen,
            },
            scale: Vector {
                value: vec2(100.0, 100.0),
                unit: Unit::Pixels,
            },
            rotation: 0.0,
        }
        .get_buffer(resolution); //Mat3::from_scale_angle_translation(vec2(0.4, 0.9), 0.0, vec2(-0.5, 0.0));
        // let transform1 = Mat3::from_scale_angle_translation(vec2(0.4, 0.9), 0.0, vec2(0.5, 0.0));
        unsafe {
            vulkano_taskgraph::execute(
                &app.queue,
                &app.resources,
                app.flight_id,
                |_cbf, tcx| {
                    *tcx.write_buffer(transform_buffer_ids[0], ..) = transform0;
                    // *tcx.write_buffer(transform_buffer_ids[1], ..) = shaders::Transform {
                    //     mat: [
                    //         transform1.x_axis.to_array().into(),
                    //         transform1.y_axis.to_array().into(),
                    //         transform1.z_axis.to_array().into(),
                    //     ],
                    // };
                    *tcx.write_buffer(local_buffer_ids[0], ..) = shaders::LocalParams {
                        col: [1.0, 0.0, 0.0],
                    };
                    // *tcx.write_buffer(local_buffer_ids[1], ..) = shaders::LocalParams {
                    //     col: [0.0, 1.0, 0.0],
                    // };
                    Ok(())
                },
                [
                    (transform_buffer_ids[0], HostAccessType::Write),
                    // (transform_buffer_ids[1], HostAccessType::Write),
                    (local_buffer_ids[0], HostAccessType::Write),
                    // (local_buffer_ids[1], HostAccessType::Write),
                ],
                [],
                [],
            )
        }
        .unwrap();

        let global_storage_buffer_id = bcx
            .global_set()
            .create_storage_buffer(buffers.global, 0, None)
            .unwrap();

        let transform_storage_buffer_ids = transform_buffer_ids
            .iter()
            .map(|&id| bcx.global_set().create_storage_buffer(id, 0, None).unwrap())
            .collect::<Vec<StorageBufferId>>();

        let local_storage_buffer_ids = local_buffer_ids
            .iter()
            .map(|&id| bcx.global_set().create_storage_buffer(id, 0, None).unwrap())
            .collect::<Vec<StorageBufferId>>();

        let push0 = shaders::PushConstants {
            global_buffer_id: global_storage_buffer_id,
            transform_buffer_id: transform_storage_buffer_ids[0],
            local_buffer_id: local_storage_buffer_ids[0],
        };
        // let push1 = shaders::PushConstants {
        //     global_buffer_id: global_storage_buffer_id,
        //     transform_buffer_id: transform_storage_buffer_ids[1],
        //     local_buffer_id: local_storage_buffer_ids[1],
        // };

        self.pass_data = Some(RenderData {
            layout: layout.clone(),
            pipelines: vec![PipelineData {
                pipeline: create_pipeline(
                    &app.device,
                    &subpass,
                    &vertex_input_state,
                    &layout.clone(),
                    &stages,
                ),
                panels: vec![
                    push0,
                    // push1
                ],
            }],
        });
    }
}

impl Task for RenderTask {
    type World = RenderContext;

    fn clear_values(&self, clear_values: &mut ClearValues<'_>, _world: &Self::World) {
        clear_values.set(self.swapchain_id.current_image_id(), [0.0, 0.0, 0.0]);
    }

    unsafe fn execute(
        &self,
        cbf: &mut RecordingCommandBuffer<'_>,
        tcx: &mut TaskContext<'_>,
        rcx: &Self::World,
    ) -> vulkano_taskgraph::TaskResult {
        unsafe {
            cbf.set_viewport(0, slice::from_ref(&rcx.viewport));
            cbf.bind_vertex_buffers(0, &[self.vertex_buffer_id], &[0], &[], &[]);
            let pass_data = self.pass_data.as_ref().unwrap();
            *tcx.write_buffer(rcx.buffers.global, ..) = shaders::GlobalParams {
                time: (Instant::now() - rcx.start_time).as_secs_f32(),
            };
            for pipeline_data in &pass_data.pipelines {
                cbf.bind_pipeline_graphics(&pipeline_data.pipeline);
                for push_constant in &pipeline_data.panels {
                    cbf.push_constants(&pass_data.layout, 0, push_constant);
                    cbf.draw(VERTICES.len() as u32, 1, 0, 0);
                }
            }
        };
        Ok(())
    }
}
