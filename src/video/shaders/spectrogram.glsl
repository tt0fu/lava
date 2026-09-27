#version 460

#include "lib/consts.glsl"
#include "lib/push_constants.glsl"
#include "lib/waveform.glsl"
#include "lib/dft.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, SpectrogramParams{
    vec4 background;
    vec4 foreground;
    float gain;
    bool debug;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    float bin = UV.x * (float(DFT.bin_count) - 1);
    float chosen_bin = dft_get_bin(waveform_sample_rate() / WAVEFORM.period);
    float val = step(1.0 - UV.y, dft_smooth_magnitude(bin) * MATERIAL.gain) + float(MATERIAL.debug) * (
                step(abs(dft_get_bin(BASS_HIGH_FREQ) - bin), 1) +
                    step(abs(dft_get_bin(LOW_MID_HIGH_FREQ) - bin), 1) +
                    step(abs(dft_get_bin(HIGH_MID_HIGH_FREQ) - bin), 1) +
                    step(abs(bin - chosen_bin), 2));
    COLOR = mix(MATERIAL.background, MATERIAL.foreground, val);
}
