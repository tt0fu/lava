use crate::video::{parameters::Write, render_context::RenderContext};
use vulkano_taskgraph::{Task, TaskContext, command_buffer::RecordingCommandBuffer};

pub struct WriteTask {}

impl Task for WriteTask {
    type World = RenderContext;

    unsafe fn execute(
        &self,
        _cbf: &mut RecordingCommandBuffer<'_>,
        tcx: &mut TaskContext<'_>,
        rcx: &Self::World,
    ) -> vulkano_taskgraph::TaskResult {
        rcx.global_parameters.write(rcx.buffers.global, tcx);
        rcx.stream.write(rcx.buffers.waveform, tcx);
        Ok(())
    }
}
