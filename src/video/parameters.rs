use std::collections::HashMap;

use anyhow::{Result, anyhow};
use vulkano::{
    buffer::{Buffer, BufferContents},
    memory::allocator::DeviceLayout,
};
use vulkano_taskgraph::{Id, TaskContext, descriptor_set::SampledImageId};

pub struct ImageIds(HashMap<String, SampledImageId>);

impl ImageIds {
    pub fn get(&self, name: &str) -> Result<SampledImageId> {
        self.0
            .get(name)
            .copied()
            .ok_or_else(|| anyhow!("unknown image '{name}'"))
    }
}

impl FromIterator<(String, SampledImageId)> for ImageIds {
    fn from_iter<I: IntoIterator<Item = (String, SampledImageId)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

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

pub trait Parameters: Layout + Write + Send + Sync {
    fn resolve_images(&self, images: &ImageIds) -> Result<()>;
}

pub trait ParametersMut: Layout + WriteMut + Send + Sync {}

pub trait TypedParameters: Send + Sync {
    type Content: BufferContents;

    fn get_content(&self) -> Self::Content;

    fn resolve_images(&self, _images: &ImageIds) -> Result<()> {
        Ok(())
    }
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

impl<T: TypedParameters> Parameters for T {
    fn resolve_images(&self, images: &ImageIds) -> Result<()> {
        TypedParameters::resolve_images(self, images)
    }
}
