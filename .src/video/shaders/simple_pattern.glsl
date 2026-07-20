#version 450

#include "lib/in_out.glsl"

#include "lib/consts.glsl"

#include "structs/pattern.glsl"

#include "uniforms/aspect_ratio.glsl"
#include "uniforms/bass.glsl"

layout(set = 0, binding = 10) uniform SimplePatternParameters {
    Pattern pattern;
};

void main() {
    vec3 col = get_color(pattern, UV, aspect_ratio, chrono);
    COLOR = vec4(col, 1.0);
}
