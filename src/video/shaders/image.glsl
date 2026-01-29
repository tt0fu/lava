#version 450

#include "lib/consts.glsl"

layout(location = 0) in vec2 UV;
layout(location = 0) out vec4 COLOR;

#include "uniforms/image.glsl"
#include "uniforms/bass.glsl"

layout(set = 0, binding = 10) uniform ImageParameters {
    float alpha_cutoff;
    float scale_min;
    float scale_max;
};

void main() {
    float scale = mix(scale_min, scale_max, bass);
    vec2 uv = (UV - vec2(0.5, 0.5)) / scale + vec2(0.5, 0.5);
    COLOR = image_color(uv);
    if (COLOR.a < alpha_cutoff) {
        discard;
    }
}
