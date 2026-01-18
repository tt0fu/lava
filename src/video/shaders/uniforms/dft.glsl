#ifndef DFT
#define DFT

#include "../consts.glsl"

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

#endif
