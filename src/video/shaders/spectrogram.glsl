#version 450

#include "lib/in_out.glsl"

#include "lib/consts.glsl"

#include "structs/pattern.glsl"

#include "uniforms/aspect_ratio.glsl"
#include "uniforms/bass.glsl"
#include "uniforms/dft.glsl"

layout(set = 0, binding = 10) uniform SpectrogramParameters {
    Pattern pattern;
    float min_frequency;
    float max_frequency;
    float gain;
    float add;
    bool circular;
};

void main() {
    vec2 p = UV - vec2(0.5);
    float r = length(p);
    float bin_norm = (circular ? abs(atan(p.x, -p.y)) / 3.1415926535 : UV.x);
    float bin = mix(frequency_to_bin(min_frequency), frequency_to_bin(max_frequency), bin_norm);
    float height = circular ? r : 1.0 - UV.y;
    float val = step(height, smooth_magnitude(bin) * gain + add);
    vec3 col = get_color(pattern, UV, aspect_ratio, chrono);
    COLOR = vec4(col, val);
}
