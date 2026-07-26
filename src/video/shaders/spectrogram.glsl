#version 460

#include "lib/push_constants.glsl"
#include "lib/waveform.glsl"
#include "lib/dft.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material, SpectrogramParams {
    vec3 col;
    float gain;
})

#define material vko_buffer(material, material_buffer_id)

void main() {
    float bin = UV.x * (float(dft.bin_count) - 1);
    float chosen_bin = get_bin(SAMPLE_RATE_F / waveform.period);
    float val = step(1.0 - UV.y, smooth_magnitude(bin) * material.gain);
    COLOR = vec4(abs(bin - chosen_bin) < 3 ? vec3(1.0, 0.0, 0.0) : material.col, val);
}
