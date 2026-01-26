#ifndef DFT
#define DFT

#include "../../../consts.glsl"

layout(set = 0, binding = 4) uniform Dft {
    vec2 dft[BIN_COUNT];
};

float smooth_magnitude(float bin) {
    return mix(
        length(dft[int(floor(bin))]),
        length(dft[int(ceil(bin))]),
        fract(bin)
    );
}

float frequency_to_bin(float frequency) {
	return clamp(EXP_BINS * log2(frequency / LOWEST_FREQUENCY), 0.0, BIN_COUNT_F);
}

float bin_to_frequency(float bin) {
	return LOWEST_FREQUENCY * exp2(bin / EXP_BINS);
}

#endif
