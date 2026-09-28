#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, GroupParams {
    SampledImageId image;
    mat3 uv_to_local;
    vec2 size;
})

#define GROUP vko_buffer(material_buffer, material_buffer_id)

void main() {
    // The synthetic panel covers the group's AABB. Reject fragments outside the group's rotated
    // rectangle, then reproduce the group's pixels 1:1. The group's local size may be negative
    // (negative scales flip a panel), so the valid range is between 0 and `size` on each axis.
    vec2 local = (GROUP.uv_to_local * vec3(UV, 1.0)).xy;
    vec2 lo = min(vec2(0.0), GROUP.size);
    vec2 hi = max(vec2(0.0), GROUP.size);
    if (any(lessThan(local, lo)) || any(greaterThan(local, hi))) {
        discard;
    }
    COLOR = texture(vko_sampler2D(GROUP.image, sampler_id), UV);
}
