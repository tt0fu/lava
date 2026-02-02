#version 450

#include "lib/consts.glsl"
#include "structs/pattern.glsl"

layout(location = 0) in vec2 UV;
layout(location = 0) out vec4 COLOR;

#include "uniforms/aspect_ratio.glsl"
#include "uniforms/samples.glsl"
#include "uniforms/stabilization.glsl"
#include "uniforms/bass.glsl"

layout(set = 0, binding = 10) uniform WaveformParameters {
    Pattern pattern;
    float line_width;
    float gain;
};

#include "lib/oklab.glsl"
#include "lib/noise.glsl"

float get_raw_sample(int sample_index) {
    if (sample_index < 0) {
        sample_index += int(period * ceil(float(-sample_index) / period));
    }
    if (sample_index >= int(SAMPLE_COUNT)) {
        sample_index -= int(period * ceil(float(sample_index - int(SAMPLE_COUNT) + 1) / period));
    }
    return samples_data[(uint(sample_index) + samples_start) % SAMPLE_COUNT] * gain;
}

float get_sample(float sample_index) {
    return mix(
        get_raw_sample(int(floor(sample_index))),
        get_raw_sample(int(ceil(sample_index))),
        fract(sample_index)
    );
}

float fade(float dist) {
    float x = clamp(dist, 0, 1);
    return 1.0 - (x * x);
}

float pseudo_cross(vec2 a, vec2 b) {
    return a.x * b.y - b.x * a.y;
}

float point_to_segment(vec2 a, vec2 b, vec2 p) {
    vec2 pa = a - p;
    vec2 pb = b - p;
    vec2 ab = b - a;
    if (dot(ab, -pa) < 0.0 || dot(-ab, -pb) < 0.0) {
        return min(length(pa), length(pb));
    }
    if (length(ab) < 1e-12) {
        return length(pa);
    }
    return abs(pseudo_cross(pa, pb)) / length(ab);
}

vec2 sample_point(float sample_index) {
    return vec2(sample_index / SAMPLE_COUNT_F * aspect_ratio,
        get_sample(sample_index) * 0.5 + 0.5);
}

float wave_distance(float sample_index, float sample_height) {
    vec2 target = vec2(sample_index / SAMPLE_COUNT_F * aspect_ratio, sample_height);
    float start_index = floor(sample_index - line_width);
    float end_index = ceil(sample_index + line_width);
    float mn = 100000.0;
    vec2 prev = sample_point(start_index);
    for (float index = start_index + 1.0; index <= end_index; index++) {
        vec2 cur = sample_point(index);
        mn = min(mn, point_to_segment(prev, cur, target));
        prev = cur;
    }
    return mn;
}

void main() {
    float sample_index = UV.x * SAMPLE_COUNT_F;
    float dist = wave_distance(sample_index + center_sample - SAMPLE_COUNT_F * focus, 1.0 - UV.y);
    float val = fade(dist * SAMPLE_COUNT_F / line_width);
    vec3 col = get_color(pattern, UV, aspect_ratio, chrono);
    COLOR = vec4(col, val);
}
