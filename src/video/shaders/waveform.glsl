#version 460

#include "lib/push_constants.glsl"
#include "lib/transform.glsl"
#include "lib/global.glsl"
#include "lib/waveform.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, WaveformParams {
    vec3 col;
    float line_width;
    float gain;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

float fade(float dist) {
    // return dist > 1.0 ? 0.0 : 1.0;
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
    return vec2(sample_index / float(WAVEFORM.sample_count) * TRANSFORM.aspect_ratio,
        waveform_get(sample_index) * MATERIAL.gain * 0.5 + 0.5);
}

float wave_distance(float sample_index, float sample_height) {
    vec2 target = vec2(sample_index / float(WAVEFORM.sample_count) * TRANSFORM.aspect_ratio, sample_height);
    float start_index = floor(sample_index - MATERIAL.line_width);
    float end_index = ceil(sample_index + MATERIAL.line_width);
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
    float sample_index = UV.x * float(WAVEFORM.sample_count);
    float dist = wave_distance(waveform_get_stabilized_index(sample_index), 1.0 - UV.y);
    float val = fade(dist * float(WAVEFORM.sample_count) / MATERIAL.line_width);
    COLOR = vec4(MATERIAL.col, val);
}
