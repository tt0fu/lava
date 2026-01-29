#version 450

#include "lib/consts.glsl"

layout(location = 0) in vec2 UV;
layout(location = 0) out vec4 COLOR;

#include "uniforms/aspect_ratio.glsl"
#include "uniforms/samples.glsl"
#include "uniforms/stabilization.glsl"
#include "uniforms/bass.glsl"

layout(set = 0, binding = 10) uniform RainbowParameters {
    float lightness;
    float chroma;
    float scale;
    float pattern_speed;
    float scroll_speed;
};

#include "lib/oklab.glsl"
#include "lib/noise.glsl"

void main() {
    COLOR = vec4(lch_srgb(vec3(lightness, chroma, fract(pattern(UV * vec2(aspect_ratio, 1.0) * scale, chrono * pattern_speed) * 2.0) + chrono * scroll_speed)), 1.0);
}
