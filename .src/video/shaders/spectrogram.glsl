#version 450

#include "lib/in_out.glsl"

#include "lib/consts.glsl"

#include "structs/pattern.glsl"

#include "uniforms/aspect_ratio.glsl"
#include "uniforms/bass.glsl"
#include "uniforms/dft.glsl"

layout(set = 0, binding = 10) uniform SpectrogramParameters {
    Pattern pattern;
    float gain;
};

void main() {
    float val = step(1.0 - UV.y, smooth_magnitude(UV.x * (BIN_COUNT - 1)) * gain);
    vec3 col = get_color(pattern, UV, aspect_ratio, chrono);
    COLOR = vec4(col, val);
}
