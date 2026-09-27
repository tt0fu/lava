use crate::video::{
    buffers::StorageBuffers,
    model::{MyVertex, VERTICES},
    render_context::RenderContext,
    scene_data::SceneData,
    shaders,
    tasks::create_pipeline::create_graphics_pipeline,
};
use std::{slice, sync::Arc};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::{Device, Queue},
    image::Image,
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
    pipeline::{
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::vertex_input::{Vertex, VertexDefinition},
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
    pub depth_buffer_id: Id<Image>,
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
        depth_buffer_id: Id<Image>,
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
            depth_buffer_id,
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

        let min_order = scene_data.panels.iter().map(|p| p.order).min().unwrap();
        let max_order = scene_data.panels.iter().map(|p| p.order).max().unwrap();

        let pushes = scene_data
            .panels
            .iter()
            .map(|p| storage_buffers.push_constants(p, min_order, max_order, sampler_id))
            .collect::<Vec<shaders::PushConstants>>();

        let mut panels_per_pipeline = vec![vec![]; scene_data.shaders.len()];

        scene_data.panels.iter().enumerate().for_each(|(ind, p)| {
            panels_per_pipeline[scene_data.materials[p.material_id].shader_id].push(ind)
        });

        let pipelines = scene_data
            .shaders
            .iter()
            .enumerate()
            .map(|(ind, e)| PipelineData {
                pipeline: create_graphics_pipeline(
                    device,
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

            if rcx.rewrite_transforms {
                for i in 0..self.scene_data.transforms.len() {
                    self.scene_data.transforms[i].write(
                        rcx.viewport.extent.into(),
                        rcx.buffers.transforms[i],
                        tcx,
                    );
                }
            }
            cbf.set_viewport(0, slice::from_ref(&rcx.viewport));
            cbf.bind_vertex_buffers(0, &[self.vertex_buffer_id], &[0], &[], &[]);

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
