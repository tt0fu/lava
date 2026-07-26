#ifndef DFT
#define DFT

#include <vulkano.glsl>

#include "consts.glsl"

VKO_DECLARE_STORAGE_BUFFER(dft, Dft {
    uint bin_count;
    float lowest_frequency;
    float exp_bins;
    vec4 bands;
    vec2 bins[];
})

#define dft vko_buffer(dft, dft_buffer_id)

float get_bin(float frequency) {
    return dft.exp_bins * log2(frequency / dft.lowest_frequency);
}

float get_frequency(float bin) {
    return dft.lowest_frequency * exp2(bin / dft.exp_bins);
}

float smooth_magnitude(float bin) {
    return mix(
        length(dft.bins[int(floor(bin))]),
        length(dft.bins[int(ceil(bin))]),
        fract(bin)
    );
}

#endif