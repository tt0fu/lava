use crate::video::{
    buffers::StorageBuffers,
    geometry::{ResolvedChild, ResolvedScene},
    model::{MyVertex, VERTICES},
    render_context::RenderContext,
    scene_data::SceneData,
    shaders,
    tasks::create_pipeline::create_graphics_pipeline,
    transform::transform_buffer,
};
use std::{slice, sync::Arc};
use vulkano::{
    buffer::Buffer,
    device::Device,
    image::Image,
    pipeline::{
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            color_blend::AttachmentBlend,
            vertex_input::{Vertex, VertexDefinition},
            viewport::Viewport,
        },
    },
    render_pass::Subpass,
    swapchain::Swapchain,
};
use vulkano_taskgraph::{
    ClearValues, Id, Task, TaskContext, command_buffer::RecordingCommandBuffer,
    descriptor_set::SamplerId,
};

/// The image a group renders into.
pub enum Target {
    Swapchain(Id<Swapchain>),
    Image(Id<Image>),
}

impl Target {
    fn image_id(&self) -> Id<Image> {
        match self {
            Target::Swapchain(id) => id.current_image_id(),
            Target::Image(id) => *id,
        }
    }
}

/// What a draw reproduces, used to recompute its transform on resize.
#[derive(Clone, Copy)]
pub enum DrawSource {
    Panel(usize),
    Group(usize),
}

/// Everything needed to draw a single panel (real or synthetic).
pub struct Draw {
    pub pipeline: Arc<GraphicsPipeline>,
    pub push_constants: shaders::PushConstants,
    pub transform_index: usize,
    pub source: DrawSource,
}

pub struct RenderData {
    pub layout: Arc<PipelineLayout>,
    /// Draws in back-to-front order.
    pub draws: Vec<Draw>,
}

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

    #[allow(clippy::too_many_arguments)]
    pub fn create_render_data(
        &mut self,
        device: &Arc<Device>,
        layout: &Arc<PipelineLayout>,
        storage_buffers: &StorageBuffers,
        scene_data: &Arc<SceneData>,
        resolved: &ResolvedScene,
        group_shader: &vulkano::shader::EntryPoint,
        subpass: &Subpass,
        sampler_id: SamplerId,
    ) {
        let vertex_shader = unsafe { shaders::load_vertex(device) }
            .unwrap()
            .entry_point("main")
            .unwrap();
        let vertex_input_state = [MyVertex::per_vertex()].definition(&vertex_shader).unwrap();

        let panel_count = resolved.panels.len();
        let material_count = scene_data.materials.len();
        let group_shader_id = scene_data.shaders.len();

        let mut cache: Vec<(usize, AttachmentBlend, Arc<GraphicsPipeline>)> = Vec::new();
        let mut draws = Vec::new();
        for child in &resolved.groups[self.group].children {
            let (shader_id, blend, material, transform_index, source) = match child {
                ResolvedChild::Panel(i) => {
                    let panel = &resolved.panels[*i];
                    (
                        scene_data.materials[panel.material].shader_id,
                        panel.blend.clone(),
                        panel.material,
                        *i,
                        DrawSource::Panel(*i),
                    )
                }
                ResolvedChild::Group(g) => (
                    group_shader_id,
                    resolved.groups[*g].blend.clone(),
                    material_count + (g - 1),
                    panel_count + (g - 1),
                    DrawSource::Group(*g),
                ),
            };

            let pipeline = match cache
                .iter()
                .find(|(shader, cached_blend, _)| *shader == shader_id && cached_blend == &blend)
            {
                Some((_, _, pipeline)) => pipeline.clone(),
                None => {
                    let fragment = if shader_id == group_shader_id {
                        group_shader
                    } else {
                        &scene_data.shaders[shader_id]
                    };
                    let pipeline = create_graphics_pipeline(
                        device,
                        subpass,
                        &vertex_input_state,
                        layout,
                        &[
                            PipelineShaderStageCreateInfo::new(&vertex_shader),
                            PipelineShaderStageCreateInfo::new(fragment),
                        ],
                        &blend,
                    );
                    cache.push((shader_id, blend, pipeline.clone()));
                    pipeline
                }
            };

            draws.push(Draw {
                pipeline,
                push_constants: storage_buffers.push_constants(
                    material,
                    transform_index,
                    sampler_id,
                ),
                transform_index,
                source,
            });
        }

        self.render_data = Some(RenderData {
            layout: layout.clone(),
            draws,
        });
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
            let pass_data = self.render_data.as_ref().unwrap();

            if rcx.rewrite_transforms {
                for draw in &pass_data.draws {
                    match draw.source {
                        DrawSource::Panel(i) => {
                            let panel = &rcx.resolved.panels[i];
                            *tcx.write_buffer(rcx.buffers.transforms[draw.transform_index], ..) =
                                transform_buffer(panel.ndc, panel.aspect_ratio);
                        }
                        DrawSource::Group(g) => {
                            let group = &rcx.resolved.groups[g];
                            *tcx.write_buffer(rcx.buffers.transforms[draw.transform_index], ..) =
                                transform_buffer(group.composite_ndc, 1.0);
                            let material = rcx.group_material[g].unwrap();
                            *tcx.write_buffer::<shaders::GroupParams>(
                                rcx.buffers.materials[material],
                                ..,
                            ) = group.params(rcx.target_sampled[g].unwrap());
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
