#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"
#include "lib/transform.glsl"
#include "lib/bands.glsl"
#include "lib/oklab.glsl"
#include "lib/noise.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, PatternParams {
    float lightness;
    float chroma;
    float scale;
    float repeats;
    float warp_speed;
    float scroll_speed;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    COLOR = vec4(
        lch_srgb(
            vec3(
                MATERIAL.lightness, 
                MATERIAL.chroma, 
                fract(
                    fbm3(
                        UV * vec2(TRANSFORM.aspect_ratio, 1.0) * MATERIAL.scale, 
                        BANDS.chrono.x * MATERIAL.warp_speed) * 
                    MATERIAL.repeats) + 
                BANDS.chrono.x * MATERIAL.scroll_speed
            )
        ), 
        1.0);
}
