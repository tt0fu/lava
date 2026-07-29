#version 460

#include "lib/push_constants.glsl"
#include "lib/waveform.glsl"
#include "lib/dft.glsl"
#include "lib/bands.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, BandsParams {
    vec3 col;
    vec4 gain;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    vec4 cur = bands_get((1.0 - UV.x) * (BANDS.history_length * BANDS.history_delta)) * MATERIAL.gain;

    float y = (1.0 - UV.y) * 4;
    int band = int(y);
    float in_band = fract(y);
    float value = 0.5;
    switch (band) {
        case 0: {
            value = cur.x;
            break;
        }
        case 1: {
            value = cur.y;
            break;
        }
        case 2: {
            value = cur.z;
            break;
        }
        case 3: {
            value = cur.w;
            break;
        }
    }
    float alpha = step(abs(0.5 - in_band), value) + value;
    COLOR = vec4(MATERIAL.col, alpha);
}
