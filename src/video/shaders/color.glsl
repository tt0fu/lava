#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"
#include "lib/bands.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, ColorParams {
    vec4 color;
    vec4 band_weights;
    vec4 band_color;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    COLOR = mix(MATERIAL.color, MATERIAL.band_color, dot(bands_get_raw(0) * MATERIAL.band_weights, vec4(1.0)));
}
