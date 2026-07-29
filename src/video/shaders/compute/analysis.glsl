#version 460

#include <vulkano.glsl>

#include "../lib/consts.glsl"
#include "../lib/compute_push_constants.glsl"
#include "../lib/global.glsl"
#include "../lib/waveform.glsl"
#include "../lib/dft.glsl"
#include "../lib/bands.glsl"

vec4 get_bands(float freq) { // (bass, low, high, treble)
    float bass = freq < BASS_HIGH_FREQ ? 1.0 : 0.0;
    float low = BASS_HIGH_FREQ < freq && freq < LOW_MID_HIGH_FREQ ? 1.0 : 0.0;
    float high = LOW_MID_HIGH_FREQ < freq && freq < HIGH_MID_HIGH_FREQ ? 1.0 : 0.0;
    float treble = HIGH_MID_HIGH_FREQ < freq ? 1.0 : 0.0;
    return vec4(bass, low, high, treble);
}

float period_bias(uint bin) {
    return exp(-10.0 * float(bin) / float(DFT.bin_count));
}

const uint MAX_BINS = 8192;

#define GROUP_SIZE 64

shared float magnitudes[MAX_BINS];
shared float candidate_scores[GROUP_SIZE];
shared uint candidate_bins[GROUP_SIZE];
shared vec4 candidate_bands[GROUP_SIZE];

layout(local_size_x = GROUP_SIZE, local_size_y = 1, local_size_z = 1) in;
void main() {
    uint id = gl_LocalInvocationID.x;

    candidate_bands[id] = vec4(-1.0);

    for (uint bin = id; bin < DFT.bin_count; bin += GROUP_SIZE) {
        float mag = length(DFT.bins[bin]);
        magnitudes[bin] = length(DFT.bins[bin]);

        vec4 bands = get_bands(dft_get_frequency(float(bin)));
        candidate_bands[id] = max(candidate_bands[id], bands * mag);
    }
    barrier();

    candidate_scores[id] = -1.0;
    candidate_bins[id] = 0;
    for (uint bin = id + 1; bin < DFT.bin_count - 1; bin += GROUP_SIZE) {
        float mag = magnitudes[bin];
        float score = mag * period_bias(bin);
        if (mag > magnitudes[bin - 1] && mag > magnitudes[bin + 1] && score > candidate_scores[id]) {
            candidate_scores[id] = score;
            candidate_bins[id] = bin;
        }
    }
    barrier();

    if (id == 0) {
        vec4 best_bands = vec4(-1.0);

        float best_score = -1.0;
        uint best_bin = 0;
        for (uint i = 0; i < GROUP_SIZE; i++) {
            best_bands = max(candidate_bands[i], best_bands);

            if (candidate_scores[i] > best_score) {
                best_score = candidate_scores[i];
                best_bin = candidate_bins[i];
            }
        }

        BANDS.start = (BANDS.start + BANDS.history_length - 1) % BANDS.history_length;
        BANDS.history[BANDS.start] = best_bands;
        BANDS.chrono += best_bands * GLOBAL.delta;

        float frequency = dft_get_frequency(float(best_bin));
        float period = SAMPLE_RATE_F / frequency;
        WAVEFORM.period = period;
        float angle = atan(DFT.bins[best_bin].y, DFT.bins[best_bin].x) / (PI * 2.0) - 0.25;
        WAVEFORM.center_sample = (angle + ceil(WAVEFORM.sample_count * WAVEFORM.focus / period)) * period;
    }
}
