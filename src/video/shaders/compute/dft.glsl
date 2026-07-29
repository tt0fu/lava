#version 460

#include <vulkano.glsl>

#include "../lib/compute_push_constants.glsl"
#include "../lib/global.glsl"
#include "../lib/waveform.glsl"
#include "../lib/dft.glsl"

const float PERIOD_COUNT = 16.0;

float window(float x, float a) {
    if (x < -1.0 || x > 1.0) {
        return 0.0;
    }
    return exp(a * sqrt(max(0.0, 1.0 - x * x))) * exp(-a);
}

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    if (idx >= DFT.bin_count) {
        return;
    }

    float bin_f = float(idx);
    float frequency = dft_get_frequency(float(idx));
    float sample_period = SAMPLE_RATE_F / frequency;
    float phase_delta = 2.0 * PI / sample_period;

    float sample_count_f = float(WAVEFORM.sample_count);

    // float window_size = min(PERIOD_COUNT * sample_period, sample_count_f);
    float window_size = sample_count_f;
    float window_start_f = floor((sample_count_f - window_size) * 0.5);
    float window_end_f = ceil((sample_count_f + window_size) * 0.5);

    int window_start = int(window_start_f);
    int window_end = int(window_end_f);
    int window_len = window_end - window_start;

    float initial_phase = phase_delta * window_start_f;

    vec2 amplitude = vec2(0.0);
    float total_window = 0.0;

    for (int i = 0; i < window_len; ++i) {
        int sample_index = window_start + i;
        float cur_sample = waveform_get_raw(sample_index);
        float x = (float(sample_index) * 2.0 - sample_count_f) / window_size;
        float w = window(x, 10.0);//frequency * frequency / 500.0);

        float phase = initial_phase + phase_delta * float(i);
        vec2 complex_exp = vec2(cos(phase), sin(phase));

        amplitude += complex_exp * cur_sample * w;
        total_window += w;
    }
    DFT.bins[idx] = amplitude / total_window;
}
