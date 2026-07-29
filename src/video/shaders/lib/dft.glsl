#ifndef LIB_DFT
#define LIB_DFT

#include <vulkano.glsl>

#include "consts.glsl"

VKO_DECLARE_STORAGE_BUFFER(dft_buffer, Dft {
    uint bin_count;
    float lowest_frequency;
    float exp_bins;
    vec2 bins[];
})

#define DFT vko_buffer(dft_buffer, dft_buffer_id)

float dft_get_bin(float frequency) {
    return DFT.exp_bins * log2(frequency / DFT.lowest_frequency);
}

float dft_get_frequency(float bin) {
    return DFT.lowest_frequency * exp2(bin / DFT.exp_bins);
}

float dft_smooth_magnitude(float bin) {
    return mix(
        length(DFT.bins[int(floor(bin))]),
        length(DFT.bins[int(ceil(bin))]),
        fract(bin)
    );
}

#endif