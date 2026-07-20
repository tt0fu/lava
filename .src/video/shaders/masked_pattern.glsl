#version 450

#include "lib/in_out.glsl"

#include "lib/consts.glsl"
#include "structs/pattern.glsl"

#include "uniforms/image.glsl"
#include "uniforms/bass.glsl"
#include "uniforms/aspect_ratio.glsl"

layout(set = 0, binding = 10) uniform MaskedPatternParameters {
    Pattern pattern;
    float scale_min;
    float scale_max;
};

void main() {
    vec2 uv = (UV - vec2(0.5, 0.5)) / mix(scale_min, scale_max, bass) + vec2(0.5, 0.5);
    vec3 col = get_color(pattern, UV, aspect_ratio, chrono);
    COLOR = vec4(col, image_color(uv).a);
}
