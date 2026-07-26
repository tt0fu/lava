use vulkano::{
    buffer::{Buffer, BufferContents},
    memory::allocator::DeviceLayout,
};
use vulkano_taskgraph::{Id, TaskContext};

pub trait LayoutStatic {
    fn layout() -> DeviceLayout;
}

pub trait Layout {
    fn layout(&self) -> DeviceLayout;
}

pub trait WriteMut {
    fn write(&mut self, id: Id<Buffer>, tcx: &mut TaskContext<'_>);
}

pub trait Write {
    fn write(&self, id: Id<Buffer>, tcx: &mut TaskContext<'_>);
}

pub trait Parameters: Layout + Write + Send + Sync {}
pub trait ParametersMut: Layout + WriteMut + Send + Sync {}

pub trait TypedParameters: Send + Sync {
    type Content: BufferContents;
    fn get_content(&self) -> Self::Content;
}

impl<T: LayoutStatic> Layout for T {
    fn layout(&self) -> DeviceLayout {
        T::layout()
    }
}

impl<T: Write> WriteMut for T {
    fn write(&mut self, id: Id<Buffer>, tcx: &mut TaskContext<'_>) {
        Write::write(self, id, tcx);
    }
}

impl<T: TypedParameters> LayoutStatic for T {
    fn layout() -> DeviceLayout {
        DeviceLayout::new_sized::<T::Content>()
    }
}

impl<T: TypedParameters> Write for T {
    fn write(&self, id: Id<Buffer>, tcx: &mut TaskContext<'_>) {
        *tcx.write_buffer(id, ..) = self.get_content();
    }
}

impl<T: TypedParameters> Parameters for T {}
