#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"
#include "lib/dft.glsl"
#include "lib/bands.glsl"
#include "lib/gridnode.glsl"
#include "lib/oklab.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, GridnodeParams {
    float lightness;
    float chroma;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

float dft_bass(float x) {
    return dft_smooth_magnitude(mix(dft_get_bin(30.0), dft_get_bin(100.0), x));
}

vec3 rainbow(float hue) {
    return lch_srgb(vec3(MATERIAL.lightness, MATERIAL.chroma, hue));
}

vec3 rainbow_chrono(float x) {
    return rainbow(fract(BANDS.chrono.x * 2.0 + x * 0.5));
}

float mover(uint id, uint channel) {
    id %= 8u;
    float x = float(id) / 8.0;
    float mag = dft_bass(x);
    Mover mover = Mover(0.5, 0.33 + clamp(mag, 0.0, 0.4), mag * 2.0, 1.0, mag * 3.0 - 0.5, rainbow_chrono(x), 0.0, 0, 0.5);
    return decode(mover, channel);
}

float parlight(uint id, uint channel) {
    id %= 8u;
    float x = float(id) / 8.0;
    float mag = dft_bass(x);
    SimpleLight parlight = SimpleLight(mag * 4.0, rainbow_chrono(x), mag * 3.0 - 0.5);
    return decode(parlight, channel);
}

float laser(uint id, uint channel) {
    id %= 8u;
    float x = float(id) / 8.0;
    float mag = dft_bass(x);
    Laser laser = Laser(1.0 - mag * 1.5, 0.0, 1.0, mag * 2.0 - 0.6, 0.0, 0.0, 0.0, rainbow_chrono(x), mag * 4.0 - 0.5, 0.0, 0.5);
    return decode(laser, channel);
}

float get_channel(uint id) {
    if (id < 13u * 16u) {
        return mover(id / 13u, id % 13u);
    }
    if (id < 13u * 16u + 5u * 16u) {
        id -= 13u * 16u;
        return parlight(id / 5u, id % 5u);
    }
    if (id < 13u * 16u + 5u * 16u + 13u * 16u) {
        id -= 13u * 16u + 5u * 16u;
        return laser(id / 13u, id % 13u);
    }
    return 0.0;
}

void main() {
    uint id = channel_id(UV, GRIDNODE_RESOLUTION);
    float val = get_channel(id);
    COLOR = vec4(val, val, val, 1.0);
}
