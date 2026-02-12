use vulkano::{
    buffer::{BufferContents, allocator::SubbufferAllocator},
    descriptor_set::WriteDescriptorSet,
};

pub fn create_write_descriptor_set<T: BufferContents>(
    buffer_allocator: &SubbufferAllocator,
    binding: u32,
    content: T,
) -> WriteDescriptorSet {
    let buffer = buffer_allocator.allocate_sized().unwrap();
    assert_eq!(buffer.size(), T::LAYOUT.head_size());
    *buffer.write().unwrap() = content;
    WriteDescriptorSet::buffer(binding, buffer)
}
