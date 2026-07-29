#version 460

#include "lib/push_constants.glsl"
#include "lib/global.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, ClockParams {
    vec3 col;
    float speed;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    float val = step(UV.x, fract(GLOBAL.time * MATERIAL.speed));
    COLOR = vec4(MATERIAL.col, val);
}
