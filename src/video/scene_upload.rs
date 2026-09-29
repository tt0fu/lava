use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    device::Queue,
    format::Format,
    image::{Image, ImageAspects, ImageCreateInfo, ImageSubresourceLayers, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
};
use vulkano_taskgraph::{
    Id,
    command_buffer::{BufferImageCopy, CopyBufferToImageInfo},
    descriptor_set::{BindlessContext, SampledImageId},
    resource::{AccessTypes, Flight, HostAccessType, ImageLayoutType, Resources},
};

use crate::{
    audio::audio_settings::AudioSettings,
    video::{
        buffers::Buffers,
        parameters::ImageIds,
        sampled_image::create_sampled_image,
        scene_data::{SceneData, SceneImage},
    },
};

/// Uploads the scene's initial buffer contents and its images in a single batch, and resolves the
/// material parameters that reference those images.
pub fn upload_scene(
    scene_data: &SceneData,
    buffers: &Buffers,
    audio_settings: &AudioSettings,
    resources: &Arc<Resources>,
    queue: &Arc<Queue>,
    flight_id: Id<Flight>,
    bcx: &BindlessContext,
) -> Vec<Id<Image>> {
    let staging_buffer_ids: Vec<_> = scene_data
        .images
        .iter()
        .map(|image| create_staging_buffer(resources, image))
        .collect();
    let image_ids: Vec<_> = scene_data
        .images
        .iter()
        .map(|image| create_image(resources, image))
        .collect();

    // Image parameters must be resolved before the material buffers are written.
    let sampled_image_ids = image_ids
        .iter()
        .map(|&image_id| create_sampled_image(resources, bcx, image_id))
        .collect::<Vec<_>>();
    bind_material_images(scene_data, &sampled_image_ids);

    let host_writes = buffers
        .materials
        .iter()
        .take(scene_data.materials.len())
        .copied()
        .map(|id| (id, HostAccessType::Write))
        .chain([
            (buffers.dft, HostAccessType::Write),
            (buffers.bands, HostAccessType::Write),
        ])
        .chain(
            staging_buffer_ids
                .iter()
                .map(|&id| (id, HostAccessType::Write)),
        );

    unsafe {
        vulkano_taskgraph::execute(
            queue,
            resources,
            flight_id,
            |cbf, tcx| {
                buffers.write_initial_data(audio_settings, scene_data, tcx);
                for (image, &staging_buffer_id) in scene_data.images.iter().zip(&staging_buffer_ids)
                {
                    tcx.write_buffer::<[u8]>(staging_buffer_id, ..)
                        .copy_from_slice(&image.data);
                }
                for ((&staging_buffer_id, &image_id), image) in staging_buffer_ids
                    .iter()
                    .zip(&image_ids)
                    .zip(&scene_data.images)
                {
                    cbf.copy_buffer_to_image(&CopyBufferToImageInfo {
                        src_buffer: staging_buffer_id,
                        dst_image: image_id,
                        regions: &[BufferImageCopy {
                            image_subresource: ImageSubresourceLayers {
                                aspects: ImageAspects::COLOR,
                                ..Default::default()
                            },
                            image_extent: [image.width, image.height, 1],
                            ..Default::default()
                        }],
                        ..Default::default()
                    });
                }
                Ok(())
            },
            host_writes,
            staging_buffer_ids
                .iter()
                .map(|&id| (id, AccessTypes::COPY_TRANSFER_READ)),
            image_ids.iter().map(|&id| {
                (
                    id,
                    AccessTypes::COPY_TRANSFER_WRITE,
                    ImageLayoutType::Optimal,
                )
            }),
        )
    }
    .unwrap();

    let mut batch = resources.create_deferred_batch();
    for &id in &staging_buffer_ids {
        batch.destroy_buffer(id);
    }
    batch.enqueue();

    image_ids
}

fn bind_material_images(scene_data: &SceneData, sampled_image_ids: &[SampledImageId]) {
    let bindings: ImageIds = scene_data
        .images
        .iter()
        .map(|image| image.name.clone())
        .zip(sampled_image_ids.iter().copied())
        .collect();
    for material in &scene_data.materials {
        material
            .parameters
            .resolve_images(&bindings)
            .expect("image parameters were validated while parsing the config");
    }
}

fn create_staging_buffer(resources: &Arc<Resources>, image: &SceneImage) -> Id<Buffer> {
    resources
        .create_buffer(
            &BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            &AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            DeviceLayout::from_size_alignment(image.data.len() as u64, 1).unwrap(),
        )
        .unwrap()
}

fn create_image(resources: &Arc<Resources>, image: &SceneImage) -> Id<Image> {
    resources
        .create_image(
            &ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [image.width, image.height, 1],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            &AllocationCreateInfo::default(),
        )
        .unwrap()
}
