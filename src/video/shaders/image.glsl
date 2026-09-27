#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, ImageParams {
    SampledImageId image;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    COLOR = texture(vko_sampler2D(MATERIAL.image, sampler_id), UV);
}
