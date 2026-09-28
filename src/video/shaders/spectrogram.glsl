#version 460

#include "lib/consts.glsl"
#include "lib/push_constants.glsl"
#include "lib/transform.glsl"
#include "lib/waveform.glsl"
#include "lib/dft.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, SpectrogramParams{
    vec4 background;
    vec4 foreground;
    float min_frequency;
    float max_frequency;
    float gain;
    float add;
    bool circular;
    bool debug;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    vec2 from_center = (UV - vec2(0.5)) * vec2(TRANSFORM.aspect_ratio, 1.0);
    float height = MATERIAL.circular ? length(from_center) : 1.0 - UV.y;
    float bin_selection = MATERIAL.circular ? abs(atan(from_center.x, -from_center.y)) / PI : UV.x;
    float bin = mix(dft_get_bin(MATERIAL.min_frequency), dft_get_bin(MATERIAL.max_frequency), bin_selection); 
    float chosen_bin = dft_get_bin(waveform_sample_rate() / WAVEFORM.period);
    float val = step(height, dft_smooth_magnitude(bin) * MATERIAL.gain + MATERIAL.add) + float(MATERIAL.debug) * (
                step(abs(dft_get_bin(BASS_HIGH_FREQ) - bin), 1) +
                    step(abs(dft_get_bin(LOW_MID_HIGH_FREQ) - bin), 1) +
                    step(abs(dft_get_bin(HIGH_MID_HIGH_FREQ) - bin), 1) +
                    step(abs(bin - chosen_bin), 2));
    COLOR = mix(MATERIAL.background, MATERIAL.foreground, val);
}
