#version 450

#include "lib/consts.glsl"
#include "structs/pattern.glsl"

layout(location = 0) in vec2 UV;
layout(location = 0) out vec4 COLOR;

#include "uniforms/aspect_ratio.glsl"
#include "uniforms/bass.glsl"

layout(set = 0, binding = 10) uniform PatternParameters {
    Pattern pattern;
};

void main() {
    vec3 col = get_color(pattern, UV, aspect_ratio, chrono);
    COLOR = vec4(col, 1.0);
}
