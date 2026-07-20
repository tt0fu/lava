#version 450

#include "lib/in_out.glsl"

#include "lib/consts.glsl"

#include "uniforms/image.glsl"
#include "uniforms/bass.glsl"

layout(set = 0, binding = 10) uniform ImageParameters {
    float scale_min;
    float scale_max;
};

void main() {
    float scale = mix(scale_min, scale_max, bass);
    vec2 uv = (UV - vec2(0.5, 0.5)) / scale + vec2(0.5, 0.5);
    COLOR = image_color(uv);
}
