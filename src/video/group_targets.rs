use std::sync::Arc;

use glam::Vec2;
use vulkano::{
    format::Format,
    image::{Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::AllocationCreateInfo,
};
use vulkano_taskgraph::{
    Id,
    descriptor_set::{BindlessContext, SampledImageId},
    graph::TaskGraph,
    resource::Resources,
};

use crate::video::{geometry::ResolvedScene, sampled_image::create_sampled_image};

/// The offscreen color targets that composite groups render into. The screen group (`0`) has no
/// target because it renders straight to the swapchain.
pub struct GroupTargets {
    virtual_ids: Vec<Option<Id<Image>>>,
    physical_ids: Vec<Option<Id<Image>>>,
    sampled_ids: Vec<Option<SampledImageId>>,
}

impl GroupTargets {
    pub fn new<W: ?Sized>(
        task_graph: &mut TaskGraph<W>,
        resources: &Arc<Resources>,
        bcx: &BindlessContext,
        resolved: &ResolvedScene,
        format: Format,
    ) -> Self {
        let group_count = resolved.groups.len();
        let mut this = Self {
            virtual_ids: vec![None; group_count],
            physical_ids: vec![None; group_count],
            sampled_ids: vec![None; group_count],
        };
        for group in 1..group_count {
            let create_info = target_image_create_info(format, resolved.groups[group].aabb_size);
            this.virtual_ids[group] = Some(task_graph.add_image(&create_info));
        }
        this.create_physical_targets(resources, bcx, resolved, format);
        this
    }

    pub fn recreate(
        &mut self,
        resources: &Arc<Resources>,
        bcx: &BindlessContext,
        resolved: &ResolvedScene,
        format: Format,
    ) {
        let mut batch = resources.create_deferred_batch();
        for group in 1..self.physical_ids.len() {
            if let Some(sampled_id) = self.sampled_ids[group].take() {
                batch.destroy_sampled_image(sampled_id);
            }
            if let Some(image_id) = self.physical_ids[group].take() {
                batch.destroy_image(image_id);
            }
        }
        batch.enqueue();

        self.create_physical_targets(resources, bcx, resolved, format);
    }

    pub fn virtual_target(&self, group: usize) -> Id<Image> {
        self.virtual_ids[group].expect("the screen group has no offscreen target")
    }

    pub fn sampled_target(&self, group: usize) -> SampledImageId {
        self.sampled_ids[group].expect("the screen group has no sampled target")
    }

    pub fn physical_mappings(&self) -> impl Iterator<Item = (Id<Image>, Id<Image>)> + '_ {
        self.virtual_ids
            .iter()
            .zip(&self.physical_ids)
            .filter_map(|(virtual_id, physical_id)| Some(((*virtual_id)?, (*physical_id)?)))
    }

    fn create_physical_targets(
        &mut self,
        resources: &Arc<Resources>,
        bcx: &BindlessContext,
        resolved: &ResolvedScene,
        format: Format,
    ) {
        for group in 1..self.physical_ids.len() {
            let create_info = target_image_create_info(format, resolved.groups[group].aabb_size);
            let image_id = resources
                .create_image(&create_info, &AllocationCreateInfo::default())
                .unwrap();
            self.physical_ids[group] = Some(image_id);
            self.sampled_ids[group] = Some(create_sampled_image(resources, bcx, image_id));
        }
    }
}

fn target_image_create_info(format: Format, size: Vec2) -> ImageCreateInfo<'static> {
    ImageCreateInfo {
        image_type: ImageType::Dim2d,
        format,
        extent: [size.x.max(1.0) as u32, size.y.max(1.0) as u32, 1],
        usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
        ..Default::default()
    }
}
