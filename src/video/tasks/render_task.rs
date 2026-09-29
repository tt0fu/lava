use crate::video::{
    model::VERTICES,
    render_context::RenderContext,
    shaders,
    tasks::draw::{DrawSource, RenderData, Target},
    transform::transform_buffer,
};
use std::{slice, sync::Arc};
use vulkano::{
    buffer::Buffer,
    pipeline::{GraphicsPipeline, graphics::viewport::Viewport},
};
use vulkano_taskgraph::{
    ClearValues, Id, Task, TaskContext, command_buffer::RecordingCommandBuffer,
};

pub struct RenderTask {
    pub vertex_buffer_id: Id<Buffer>,
    pub group: usize,
    pub target: Target,
    pub background: [f32; 4],
    pub render_data: Option<RenderData>,
}

impl RenderTask {
    pub fn new(
        vertex_buffer_id: Id<Buffer>,
        group: usize,
        target: Target,
        background: [f32; 4],
    ) -> Self {
        Self {
            vertex_buffer_id,
            group,
            target,
            background,
            render_data: None,
        }
    }
}

impl Task for RenderTask {
    type World = RenderContext;

    fn clear_values(&self, clear_values: &mut ClearValues<'_>, _world: &Self::World) {
        clear_values.set(self.target.image_id(), self.background);
    }

    unsafe fn execute(
        &self,
        cbf: &mut RecordingCommandBuffer<'_>,
        tcx: &mut TaskContext<'_>,
        rcx: &Self::World,
    ) -> vulkano_taskgraph::TaskResult {
        unsafe {
            let render_data = self.render_data.as_ref().unwrap();

            if rcx.rewrite_transforms {
                for draw in &render_data.draws {
                    match draw.source {
                        DrawSource::Panel(index) => {
                            let panel = &rcx.resolved.panels[index];
                            *tcx.write_buffer(rcx.buffers.transforms[draw.transform_index], ..) =
                                transform_buffer(panel.ndc, panel.aspect_ratio);
                        }
                        DrawSource::Group(group) => {
                            let resolved_group = &rcx.resolved.groups[group];
                            *tcx.write_buffer(rcx.buffers.transforms[draw.transform_index], ..) =
                                transform_buffer(resolved_group.composite_ndc, 1.0);
                            let material = rcx.buffers.group_material(group);
                            *tcx.write_buffer::<shaders::GroupParams>(
                                rcx.buffers.materials[material],
                                ..,
                            ) = resolved_group.params(rcx.group_targets.sampled_target(group));
                        }
                    }
                }
            }

            let size = rcx.resolved.groups[self.group].aabb_size;
            let viewport = Viewport {
                offset: [0.0, 0.0],
                extent: size.to_array().into(),
                min_depth: 0.0,
                max_depth: 1.0,
            };
            cbf.set_viewport(0, slice::from_ref(&viewport));
            cbf.bind_vertex_buffers(0, &[self.vertex_buffer_id], &[0], &[], &[]);

            let mut bound_pipeline: Option<&Arc<GraphicsPipeline>> = None;
            for draw in &render_data.draws {
                if bound_pipeline.is_none_or(|pipeline| !Arc::ptr_eq(pipeline, &draw.pipeline)) {
                    cbf.bind_pipeline(&draw.pipeline);
                    bound_pipeline = Some(&draw.pipeline);
                }
                cbf.push_constants(&render_data.layout, 0, &draw.push_constants);
                cbf.draw(VERTICES.len() as u32, 1, 0, 0);
            }
        };
        Ok(())
    }
}
