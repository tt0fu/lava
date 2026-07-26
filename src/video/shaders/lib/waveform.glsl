#ifndef WAVEFORM
#define WAVEFORM

#include <vulkano.glsl>

#include "consts.glsl"

VKO_DECLARE_STORAGE_BUFFER(waveform, Waveform {
    uint sample_count;
    uint start;
    float focus;
    
    float period;
    float center_sample;
    float samples[];
})

#define waveform vko_buffer(waveform, waveform_buffer_id)

float get_raw_sample(int index) {
    return waveform.samples[(uint(index) + waveform.start) % waveform.sample_count];
}

float get_sample(int index) {
    if (index < 0) {
        index += int(waveform.period * ceil(float(-index) / waveform.period));
    }
    if (index >= int(waveform.sample_count)) {
        index -= int(waveform.period * ceil(float(index - int(waveform.sample_count) + 1) / waveform.period));
    }
    return get_raw_sample(index);
}

float get_sample(float index) {
    return mix(
        get_sample(int(floor(index))),
        get_sample(int(ceil(index))),
        fract(index)
    );
}

float get_stabilized_index(float index) {
    return index + waveform.center_sample - float(waveform.sample_count) * waveform.focus;
}

#endif
