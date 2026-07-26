#version 460

#include "lib/push_constants.glsl"
#include "lib/global.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material, ClockParams {
    vec3 col;
    float speed;
})

#define material vko_buffer(material, material_buffer_id)

void main() {
    float val = step(UV.x, fract(global.time * material.speed));
    COLOR = vec4(material.col, val);
}
