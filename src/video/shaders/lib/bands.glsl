#ifndef LIB_BANDS
#define LIB_BANDS

#include <vulkano.glsl>

#include "consts.glsl"

VKO_DECLARE_STORAGE_BUFFER(bands_buffer, Bands {
    uint history_length;
    float history_delta;
    vec4 chrono;
    uint start;
    vec4 history[];
})

#define BANDS vko_buffer(bands_buffer, bands_buffer_id)

vec4 bands_get_raw(int index) {
    return BANDS.history[(uint(clamp(index, 0, BANDS.history_length - 1)) + BANDS.start) % BANDS.history_length];
}

vec4 bands_get(float delay) {
    float index = delay / BANDS.history_delta;
    return mix(
        bands_get_raw(int(floor(index))),
        bands_get_raw(int(ceil(index))),
        fract(index)
    );
}

#endif
