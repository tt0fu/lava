use std::sync::Arc;

use vulkano::{
    device::Device,
    image::Image,
    pipeline::{
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            color_blend::AttachmentBlend,
            vertex_input::{Vertex, VertexDefinition},
        },
    },
    render_pass::Subpass,
    shader::EntryPoint,
    swapchain::Swapchain,
};
use vulkano_taskgraph::{Id, descriptor_set::SamplerId};

use crate::video::{
    buffers::Buffers,
    geometry::{ResolvedChild, ResolvedScene},
    model::MyVertex,
    pipeline::{create_graphics_pipeline, load_vertex_shader},
    scene_data::SceneData,
    shaders,
    storage_buffers::StorageBuffers,
};

pub enum Target {
    Swapchain(Id<Swapchain>),
    Image(Id<Image>),
}

impl Target {
    pub fn image_id(&self) -> Id<Image> {
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

/// The full back-to-front draw list of one group, with a single shared pipeline layout.
pub struct RenderData {
    pub layout: Arc<PipelineLayout>,
    pub draws: Vec<Draw>,
}

impl RenderData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &Arc<Device>,
        layout: &Arc<PipelineLayout>,
        buffers: &Buffers,
        storage_buffers: &StorageBuffers,
        scene_data: &Arc<SceneData>,
        resolved: &ResolvedScene,
        group: usize,
        group_shader: &EntryPoint,
        subpass: &Subpass,
        sampler_id: SamplerId,
    ) -> Self {
        let vertex_shader = load_vertex_shader(device);
        let vertex_input_state = [MyVertex::per_vertex()].definition(&vertex_shader).unwrap();
        let group_shader_id = scene_data.shaders.len();
        let panel_count = resolved.panels.len();

        let mut pipeline_cache: Vec<(usize, AttachmentBlend, Arc<GraphicsPipeline>)> = Vec::new();
        let mut draws = Vec::new();
        for child in &resolved.groups[group].children {
            let (shader_id, blend, material, transform_index, source) = match child {
                ResolvedChild::Panel(index) => {
                    let panel = &resolved.panels[*index];
                    (
                        scene_data.materials[panel.material].shader_id,
                        panel.blend.clone(),
                        panel.material,
                        *index,
                        DrawSource::Panel(*index),
                    )
                }
                ResolvedChild::Group(child_group) => (
                    group_shader_id,
                    resolved.groups[*child_group].blend.clone(),
                    buffers.group_material(*child_group),
                    panel_count + (child_group - 1),
                    DrawSource::Group(*child_group),
                ),
            };

            let pipeline = match pipeline_cache
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
                    pipeline_cache.push((shader_id, blend, pipeline.clone()));
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

        Self {
            layout: layout.clone(),
            draws,
        }
    }
}
