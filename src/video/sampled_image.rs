use std::sync::Arc;

use vulkano::image::{
    Image, ImageLayout,
    sampler::{Filter, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
    view::ImageViewCreateInfo,
};
use vulkano_taskgraph::{
    Id,
    descriptor_set::{BindlessContext, SampledImageId, SamplerId},
    resource::Resources,
};

pub fn create_sampler(bcx: &BindlessContext) -> SamplerId {
    bcx.global_set()
        .create_sampler(&SamplerCreateInfo {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            mipmap_mode: SamplerMipmapMode::Linear,
            address_mode: [SamplerAddressMode::ClampToEdge; 3],
            max_lod: None,
            ..Default::default()
        })
        .unwrap()
}

pub fn create_sampled_image(
    resources: &Arc<Resources>,
    bcx: &BindlessContext,
    image_id: Id<Image>,
) -> SampledImageId {
    let image_state = resources.image(image_id);
    let create_info = ImageViewCreateInfo::from_image(image_state.image());
    bcx.global_set()
        .create_sampled_image(image_id, &create_info, ImageLayout::ShaderReadOnlyOptimal)
        .unwrap()
}
