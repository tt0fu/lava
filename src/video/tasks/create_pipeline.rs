use std::{slice, sync::Arc};
use vulkano::{
    device::Device,
    pipeline::{
        ComputePipeline, DynamicState, GraphicsPipeline, PipelineLayout,
        PipelineShaderStageCreateInfo,
        compute::ComputePipelineCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            input_assembly::{InputAssemblyState, PrimitiveTopology::TriangleStrip},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::VertexInputState,
            viewport::ViewportState,
        },
    },
    render_pass::Subpass,
    shader::EntryPoint,
};
use vulkano_taskgraph::descriptor_set::BindlessContext;

pub fn create_compute_pipeline(
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

pub fn create_graphics_pipeline(
    device: &Arc<Device>,
    subpass: &Subpass,
    vertex_input_state: &VertexInputState,
    layout: &Arc<PipelineLayout>,
    stages: &[PipelineShaderStageCreateInfo],
    blend: &AttachmentBlend,
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
            depth_stencil_state: None,
            multisample_state: Some(&MultisampleState::default()),
            color_blend_state: Some(&ColorBlendState {
                attachments: &[ColorBlendAttachmentState {
                    blend: Some(blend.clone()),
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
