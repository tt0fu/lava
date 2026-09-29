use crate::{
    audio::audio_settings::AudioSettings,
    video::{
        pipeline::create_compute_pipeline,
        render_context::RenderContext,
        shaders::{self, ComputePushConstants},
    },
};
use std::sync::Arc;
use vulkano::{device::Device, pipeline::ComputePipeline};
use vulkano_taskgraph::{
    Task, TaskContext, command_buffer::RecordingCommandBuffer, descriptor_set::BindlessContext,
};

pub struct AnalysisTask {
    pub audio_settings: Arc<AudioSettings>,
    pub pipeline: Arc<ComputePipeline>,
    pub compute_push_constants: ComputePushConstants,
}

impl AnalysisTask {
    pub fn new(
        audio_settings: &Arc<AudioSettings>,
        device: &Arc<Device>,
        bcx: &BindlessContext,
        compute_push_constants: ComputePushConstants,
    ) -> Self {
        Self {
            audio_settings: audio_settings.clone(),
            pipeline: create_compute_pipeline(
                device,
                bcx,
                &unsafe { shaders::load_analysis(device) }
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
            ),
            compute_push_constants,
        }
    }
}

impl Task for AnalysisTask {
    type World = RenderContext;

    unsafe fn execute(
        &self,
        cbf: &mut RecordingCommandBuffer<'_>,
        _tcx: &mut TaskContext<'_>,
        _rcx: &Self::World,
    ) -> vulkano_taskgraph::TaskResult {
        unsafe {
            cbf.push_constants(self.pipeline.layout(), 0, &self.compute_push_constants);
            cbf.bind_pipeline(&self.pipeline);
            cbf.dispatch([1, 1, 1]);
        };
        Ok(())
    }
}
