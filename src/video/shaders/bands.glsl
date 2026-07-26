#version 460

#include "lib/push_constants.glsl"
#include "lib/waveform.glsl"
#include "lib/dft.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material, BandsParams {
    vec3 col;
    float gain;
})

#define material vko_buffer(material, material_buffer_id)

void main() {
    int band = int(UV.x * 4);
    
    float height = 0.5;
    switch (band) {
        case 0: {
            height = dft.bands.x;
            break;
        }
        case 1: {
            height = dft.bands.y;
            break;
        }
        case 2: {
            height = dft.bands.z;
            break;
        }
        case 3: {
            height = dft.bands.w;
            break;
        }
    }
    float val = step(1.0 - UV.y, height * material.gain);
    COLOR = vec4(material.col, val);
}
