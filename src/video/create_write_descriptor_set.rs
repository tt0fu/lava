use anyhow::Result;
use vulkano::{
    buffer::{BufferContents, allocator::SubbufferAllocator},
    descriptor_set::WriteDescriptorSet,
};

pub fn create_write_descriptor_set<T: BufferContents>(
    buffer_allocator: &SubbufferAllocator,
    binding: u32,
    content: T,
) -> Result<WriteDescriptorSet> {
    let buffer = buffer_allocator.allocate_sized()?;
    *buffer.write()? = content;
    Ok(WriteDescriptorSet::buffer(binding, buffer))
}
