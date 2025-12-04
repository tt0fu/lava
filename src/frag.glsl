#version 450

const uint SAMPLE_COUNT = 4096u;
const uint BIN_COUNT = 512u;
const float SAMPLE_COUNT_F = float(SAMPLE_COUNT);
const float BIN_COUNT_F = float(BIN_COUNT);
const float SAMPLE_RATE = 44100.0;
const float LOWEST_FREQUENCY = SAMPLE_RATE / SAMPLE_COUNT_F;
const float EXP_BINS = floor(BIN_COUNT_F / log2(SAMPLE_RATE / (2.0 * LOWEST_FREQUENCY)));

layout(location = 0) in vec2 UV;
layout(location = 0) out vec4 COLOR;

layout(set = 0, binding = 0) uniform Data {
    float scale_x;
    uint samples_start;
    float samples_data[SAMPLE_COUNT];
    float period;
    float focus;
    float center_sample;

    float line_width;
};

float get_raw_sample(int sample_index) {
    if (sample_index < 0) {
        // sample_index += int(period * ceil(float(-sample_index) / period));
        sample_index = 0;
    }
    if (sample_index >= int(SAMPLE_COUNT)) {
        // sample_index -= int(period * ceil(float(sample_index - int(SAMPLE_COUNT) + 1) / period));
        sample_index = int(SAMPLE_COUNT) - 1;
    }
    return samples_data[(uint(sample_index) + samples_start) % SAMPLE_COUNT];
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
    return vec2(sample_index / SAMPLE_COUNT_F * scale_x,
        get_sample(sample_index) * 0.5 + 0.5);
}

float wave_distance(float sample_index, float sample_height) {
    vec2 target = vec2(sample_index / SAMPLE_COUNT_F * scale_x, sample_height);
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
    float sample_index = UV.x * float(SAMPLE_COUNT);
    float dist = wave_distance(sample_index, 1.0 - UV.y); // + center_sample - float(SAMPLE_COUNT) * focus
    float val = fade(dist * float(SAMPLE_COUNT) / line_width);
    //float val = step(abs(1.0 - UV.y - bass), 0.001);
    // vec3 col = lch_srgb(vec3(0.8, 0.1, fract(pattern(UV * vec2(scale_x, 1.0), chrono) * 2.0) + chrono * 2.0));
    //COLOR = vec4(lch_srgb(vec3(0.7, 0.1, pattern(UV * 2.0, chrono))), 1.0);
    COLOR = vec4(val, val, val, 1.0);
}
