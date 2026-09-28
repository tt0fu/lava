#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, GroupParams {
    SampledImageId image;
    mat3 uv_to_aabb;
})

#define GROUP vko_buffer(material_buffer, material_buffer_id)

void main() {
    // The synthetic panel is the group's own (possibly rotated) rectangle, and `uv_to_aabb`
    // remaps its UV to the group target's AABB UV. This is a 1:1 fetch of what was rasterized
    // (the same affine map was used to draw into the target), so no fragment needs clipping.
    vec2 uv = (GROUP.uv_to_aabb * vec3(UV, 1.0)).xy;
    COLOR = texture(vko_sampler2D(GROUP.image, sampler_id), uv);
}
