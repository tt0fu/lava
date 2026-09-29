use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::Queue,
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
};
use vulkano_taskgraph::{
    Id,
    resource::{Flight, HostAccessType, Resources},
};

use crate::video::model::{MyVertex, VERTICES};

pub fn create_vertex_buffer(
    resources: &Arc<Resources>,
    queue: &Arc<Queue>,
    flight_id: Id<Flight>,
) -> Id<Buffer> {
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
    vertex_buffer_id
}
