#ifndef LIB_WAVEFORM
#define LIB_WAVEFORM

#include <vulkano.glsl>

#include "consts.glsl"

VKO_DECLARE_STORAGE_BUFFER(waveform_buffer, Waveform {
    uint sample_count;
    uint start;
    float focus;
    
    float period;
    float center_sample;
    float samples[];
})

#define WAVEFORM vko_buffer(waveform_buffer, waveform_buffer_id)

float waveform_get_raw(int index) {
    return WAVEFORM.samples[(uint(index) + WAVEFORM.start) % WAVEFORM.sample_count];
}

float waveform_get(int index) {
    if (index < 0) {
        index += int(WAVEFORM.period * ceil(float(-index) / WAVEFORM.period));
    }
    if (index >= int(WAVEFORM.sample_count)) {
        index -= int(WAVEFORM.period * ceil(float(index - int(WAVEFORM.sample_count) + 1) / WAVEFORM.period));
    }
    return waveform_get_raw(index);
}

float waveform_get(float index) {
    return mix(
        waveform_get(int(floor(index))),
        waveform_get(int(ceil(index))),
        fract(index)
    );
}

float waveform_get_stabilized_index(float index) {
    return index + WAVEFORM.center_sample - float(WAVEFORM.sample_count) * WAVEFORM.focus;
}

#endif
