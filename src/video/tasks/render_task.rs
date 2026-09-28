use crate::video::{
    buffers::StorageBuffers,
    model::{MyVertex, VERTICES},
    render_context::RenderContext,
    scene_data::{ResolvedPanel, SceneData},
    shaders,
    tasks::create_pipeline::create_graphics_pipeline,
    transform::transform_buffer,
};
use std::{slice, sync::Arc};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::{Device, Queue},
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
    pipeline::{
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            color_blend::AttachmentBlend,
            vertex_input::{Vertex, VertexDefinition},
        },
    },
    render_pass::Subpass,
    swapchain::Swapchain,
};
use vulkano_taskgraph::{
    ClearValues, Id, Task, TaskContext,
    command_buffer::RecordingCommandBuffer,
    descriptor_set::{BindlessContext, SamplerId},
    resource::{Flight, HostAccessType, Resources},
};

// Everything needed to draw a single panel
pub struct Draw {
    pub pipeline: Arc<GraphicsPipeline>,
    pub push_constants: shaders::PushConstants,
}

// Information needed to draw all panels
pub struct RenderData {
    pub layout: Arc<PipelineLayout>,
    /// Panels in draw order, back to front.
    pub draws: Vec<Draw>,
}

pub struct RenderTask {
    pub vertex_buffer_id: Id<Buffer>,
    pub swapchain_id: Id<Swapchain>,
    pub scene_data: Arc<SceneData>,
    pub render_data: Option<RenderData>,
}

impl RenderTask {
    pub fn new(
        resources: &Arc<Resources>,
        queue: &Arc<Queue>,
        flight_id: Id<Flight>,
        scene_data: &Arc<SceneData>,
        swapchain_id: Id<Swapchain>,
    ) -> Self {
        let vertex_buffer_id = resources
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
                queue,
                resources,
                flight_id,
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
            scene_data: scene_data.clone(),
            render_data,
        }
    }

    pub fn create_render_data(
        &mut self,
        device: &Arc<Device>,
        bcx: &BindlessContext,
        storage_buffers: &StorageBuffers,
        scene_data: &Arc<SceneData>,
        resolved: &[ResolvedPanel],
        subpass: &Subpass,
        sampler_id: SamplerId,
    ) {
        let vertex_shader = unsafe { shaders::load_vertex(device) }
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = [MyVertex::per_vertex()].definition(&vertex_shader).unwrap();

        let all_stages = std::iter::once(PipelineShaderStageCreateInfo::new(&vertex_shader))
            .chain(
                scene_data
                    .shaders
                    .iter()
                    .map(|e| PipelineShaderStageCreateInfo::new(&e)),
            )
            .collect::<Vec<PipelineShaderStageCreateInfo>>();

        let layout = bcx
            .pipeline_layout_from_stages(all_stages.as_slice())
            .unwrap();

        let pushes = resolved
            .iter()
            .enumerate()
            .map(|(i, p)| storage_buffers.push_constants(p, i, sampler_id))
            .collect::<Vec<shaders::PushConstants>>();

        let mut pipeline_cache: Vec<(usize, AttachmentBlend, Arc<GraphicsPipeline>)> = Vec::new();
        let mut panel_pipelines = Vec::with_capacity(resolved.len());
        for panel in resolved {
            let shader_id = scene_data.materials[panel.material].shader_id;
            let pipeline = match pipeline_cache
                .iter()
                .find(|(cached_shader, cached_blend, _)| {
                    *cached_shader == shader_id && cached_blend == &panel.blend
                }) {
                Some((_, _, pipeline)) => pipeline.clone(),
                None => {
                    let pipeline = create_graphics_pipeline(
                        device,
                        &subpass,
                        &vertex_input_state,
                        &layout,
                        &[
                            PipelineShaderStageCreateInfo::new(&vertex_shader),
                            PipelineShaderStageCreateInfo::new(&scene_data.shaders[shader_id]),
                        ],
                        &panel.blend,
                    );
                    pipeline_cache.push((shader_id, panel.blend.clone(), pipeline.clone()));
                    pipeline
                }
            };
            panel_pipelines.push(pipeline);
        }

        let draws = (0..resolved.len())
            .map(|i| Draw {
                pipeline: panel_pipelines[i].clone(),
                push_constants: pushes[i],
            })
            .collect();

        self.render_data = Some(RenderData { layout, draws });
    }
}

impl Task for RenderTask {
    type World = RenderContext;

    fn clear_values(&self, clear_values: &mut ClearValues<'_>, _world: &Self::World) {
        let bg: [f32; 4] = self.scene_data.background_color.into();
        clear_values.set(self.swapchain_id.current_image_id(), bg);
    }

    unsafe fn execute(
        &self,
        cbf: &mut RecordingCommandBuffer<'_>,
        tcx: &mut TaskContext<'_>,
        rcx: &Self::World,
    ) -> vulkano_taskgraph::TaskResult {
        unsafe {
            let pass_data = self.render_data.as_ref().unwrap();

            if rcx.rewrite_transforms {
                for (i, panel) in rcx.resolved.iter().enumerate() {
                    *tcx.write_buffer(rcx.buffers.transforms[i], ..) =
                        transform_buffer(panel.transform, panel.aspect_ratio);
                }
            }
            cbf.set_viewport(0, slice::from_ref(&rcx.viewport));
            cbf.bind_vertex_buffers(0, &[self.vertex_buffer_id], &[0], &[], &[]);

            let mut bound_pipeline: Option<&Arc<GraphicsPipeline>> = None;
            for draw in &pass_data.draws {
                if bound_pipeline.is_none_or(|p| !Arc::ptr_eq(p, &draw.pipeline)) {
                    cbf.bind_pipeline(&draw.pipeline);
                    bound_pipeline = Some(&draw.pipeline);
                }
                cbf.push_constants(&pass_data.layout, 0, &draw.push_constants);
                cbf.draw(VERTICES.len() as u32, 1, 0, 0);
            }
        };
        Ok(())
    }
}
