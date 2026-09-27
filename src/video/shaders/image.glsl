#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"
#include "lib/bands.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, ImageParams {
    SampledImageId image;
    vec4 multiply;
    vec4 add;
    vec4 band_weights;
    float min_size;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    float scale = MATERIAL.min_size + dot(bands_get_raw(0) * MATERIAL.band_weights, vec4(1.0));
    
    vec2 uv = (UV - vec2(0.5, 0.5)) / scale + vec2(0.5, 0.5);
    vec4 col = texture(vko_sampler2D(MATERIAL.image, sampler_id), uv);
    col = col * MATERIAL.multiply + MATERIAL.add;
    COLOR = col;
}
