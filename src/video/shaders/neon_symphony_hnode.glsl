#version 450

#include "lib/in_out.glsl"

#include "lib/consts.glsl"

#include "uniforms/dft.glsl"
#include "uniforms/bass.glsl"

layout(set = 0, binding = 10) uniform NeonSymphonyHnodeParameters {
    float lightness;
    float chroma;
};

#include "lib/gridnode.glsl"
#include "lib/oklab.glsl"

float dft_bass(float x) {
    return smooth_magnitude(mix(frequency_to_bin(30.0), frequency_to_bin(100.0), x));
}

vec3 rainbow(float hue) {
    return lch_srgb(vec3(lightness, chroma, hue));
}

vec3 rainbow_chrono(float x) {
    return rainbow(fract(chrono * 2.0 + x));
}

const uint WALL_TRUSS_ROPES_START = FROM_DMX(1, 1);
const uint WALL_TRUSS_ROPES_END = FROM_DMX(1, 32);

const uint CEILING_SCREEN_ROPES_START = FROM_DMX(1, 33);
const uint CEILING_SCREEN_ROPES_END = FROM_DMX(1, 57);

const uint DIAMOND_SCREEN_ROPES_START = FROM_DMX(1, 58);
const uint DIAMOND_SCREEN_ROPES_END = FROM_DMX(1, 97);

const uint REAR_MAIN = FROM_DMX(1, 98);
const uint REAR_TOP = FROM_DMX(1, 103);

const uint FORCE_RESET_TRUSS_POSITIONS = FROM_DMX(1, 108);
const uint AUDIOLINK_THEME_COLORS = FROM_DMX(1, 109);

const uint TRUSS_SPOTS_1_START = FROM_DMX(1, 140);
const uint TRUSS_SPOTS_1_END = FROM_DMX(1, 503);
const uint TRUSS_SPOTS_2_START = FROM_DMX(2, 1);
const uint TRUSS_SPOTS_2_END = FROM_DMX(2, 52);

const uint TRUSS_LIGHTBARS_START = FROM_DMX(2, 53);
const uint TRUSS_LIGHTBARS_END = FROM_DMX(2, 484);

const uint DIAMOND_BLINDERS_2_START = FROM_DMX(2, 485);
const uint DIAMOND_BLINDERS_2_END = FROM_DMX(2, 509);
const uint DIAMOND_BLINDERS_3_START = FROM_DMX(3, 1);
const uint DIAMOND_BLINDERS_3_END = FROM_DMX(3, 175);

const uint DIAMOND_SPOTS_3_START = FROM_DMX(3, 176);
const uint DIAMOND_SPOTS_3_END = FROM_DMX(3, 500);
const uint DIAMOND_SPOTS_4_START = FROM_DMX(4, 1);
const uint DIAMOND_SPOTS_4_END = FROM_DMX(4, 195);

const uint DIAMOND_LIGHTBARS_4_START = FROM_DMX(4, 196);
const uint DIAMOND_LIGHTBARS_4_END = FROM_DMX(4, 501);
const uint DIAMOND_LIGHTBARS_5_START = FROM_DMX(5, 1);
const uint DIAMOND_LIGHTBARS_5_END = FROM_DMX(5, 414);

const uint CEILING_SCREEN_POSITIONERS_START = FROM_DMX(5, 415);
const uint CEILING_SCREEN_POSITIONERS_END = FROM_DMX(5, 449);

const uint DIAMOND_SCREEN_POSITIONERS_START = FROM_DMX(5, 450);
const uint DIAMOND_SCREEN_POSITIONERS_END = FROM_DMX(5, 505);

const uint TRUSS_POSITIONERS_5_START = FROM_DMX(5, 506);
const uint TRUSS_POSITIONERS_5_END = FROM_DMX(5, 511);
const uint TRUSS_POSITIONERS_6_START = FROM_DMX(6, 1);
const uint TRUSS_POSITIONERS_6_END = FROM_DMX(6, 10);

bool inside(inout uint id, uint start, uint end) {
    if (start <= id && id <= end) {
        id -= start;
        return true;
    }
    return false;
}

bool inside(inout uint id, uint start1, uint end1, uint start2, uint end2) {
    bool ans = start1 <= id && id <= end1 || start2 <= id && id <= end2;
    if (start1 <= id && id <= end1 || start2 <= id && id <= end2) {
        id = id <= end1 ? id - start1 : id - start2;
        return true;
    }
    return false;
}

float get_channel_value(uint id) {
    if (inside(id, WALL_TRUSS_ROPES_START, WALL_TRUSS_ROPES_END)) {
        uint channel = id % 4;
        id /= 4;
        return decode(
            TripleRopeRig(
                false, // take control
                0.5, // rope 1
                0.5, // rope 2
                0.5 // rope 3
            ),
            channel
        );
    }
    if (inside(id, CEILING_SCREEN_ROPES_START, CEILING_SCREEN_ROPES_END)) {
        uint channel = id % 5;
        id /= 5;
        return decode(
            QuadRopeRig(
                false, // take control
                0.5, // rope 1
                0.5, // rope 2
                0.5, // rope 3
                0.5 // rope 4
            ),
            channel
        );
    }
    if (inside(id, DIAMOND_SCREEN_ROPES_START, DIAMOND_SCREEN_ROPES_END)) {
        uint channel = id % 5;
        id /= 5;
        return decode(
            QuadRopeRig(
                false, // take control
                0.5, // rope 1
                0.5, // rope 2
                0.5, // rope 3
                0.5 // rope 4
            ),
            channel
        );
    }
    if (inside(id, REAR_MAIN, REAR_MAIN + 4)) {
        return decode(
            QuadRopeRig(
                false, // take control
                0.5, // rope 1
                0.5, // rope 2
                0.5, // rope 3
                0.5 // rope 4
            ),
            id
        );
    }
    if (inside(id, REAR_TOP, REAR_TOP + 4)) {
        return decode(
            QuadRopeRig(
                false, // take control
                0.5, // rope 1
                0.5, // rope 2
                0.5, // rope 3
                0.5 // rope 4
            ),
            id
        );
    }
    if (id == FORCE_RESET_TRUSS_POSITIONS) {
        return 0.0; // 1.0 to reset
    }
    if (inside(id, AUDIOLINK_THEME_COLORS, AUDIOLINK_THEME_COLORS + 12)) {
        return decode(
            AudiolinkThemeColorControl(
                true, // enable
                rainbow_chrono(0.0), // color 1
                rainbow_chrono(0.25), // color 2
                rainbow_chrono(0.5), // color 3
                rainbow_chrono(0.75) // color 4
            ),
            id
        );
    }
    if (inside(id, TRUSS_SPOTS_1_START, TRUSS_SPOTS_1_END, TRUSS_SPOTS_2_START, TRUSS_SPOTS_2_END)) {
        uint channel = id % 13;
        id /= 13;
        uint in_truss = 3 - id % 4;
        return decode(
            Mover(
                0.0, // pan
                sin(6.2831 * (chrono + float(id) / 32.0)) * 0.25 + 0.5, // tilt
                1.0, // zoom
                clamp(bass * 4.0 - float(in_truss), 0.0, 1.0), // dimmer
                0.0, // strobe
                rainbow_chrono(float(id) / 32.0), // color
                bass, // gobo speed
                0, // gobo
                0.5 // speed
            ),
            channel
        );
    }
    if (inside(id, TRUSS_LIGHTBARS_START, TRUSS_LIGHTBARS_END)) {
        uint channel = id % 18;
        id /= 18;

        uint in_truss = 2 - id % 3;
        float[12] dimmers;
        for (int i = 0; i < 12; i++) { // bottom to top
            dimmers[i] = clamp(bass * 36.0 - float(in_truss * 12 + i), 0.0, 1.0);
        }
        return decode(
            TiltingLightbar(
                sin(6.2831 * (chrono + float(id) / 24.0)) * 0.25 + 0.5, // tilt
                rainbow_chrono(float(id) / 24.0), // color
                clamp(10.0 * (bass - 1.0) + 1.0, 0.0, 1.0), // strobe
                dimmers // dimmers
            ),
            channel
        );
    }
    if (inside(id, DIAMOND_BLINDERS_2_START, DIAMOND_BLINDERS_2_END, DIAMOND_BLINDERS_3_START, DIAMOND_BLINDERS_3_END)) {
        uint channel = id % 5;
        id /= 5;
        return decode(
            SimpleLight(
                bass, /// 2.0, // dimmer
                rainbow_chrono(float(id) / 40.0), // color
                clamp(10.0 * (bass - 1.0) + 1.0, 0.0, 1.0) // strobe
            ),
            channel
        );
    }
    if (inside(id, DIAMOND_SPOTS_3_START, DIAMOND_SPOTS_3_END, DIAMOND_SPOTS_4_START, DIAMOND_SPOTS_4_END)) {
        uint channel = id % 13;
        id /= 13;
        return decode(
            Mover(
                0.5, // pan
                0.5, // tilt
                1.0, // zoom
                bass, // dimmer
                clamp(10.0 * (bass - 1.0) + 1.0, 0.0, 1.0), // strobe
                rainbow_chrono(float(id) / 40.0), // color
                bass, // gobo speed
                0, // gobo
                0.5 // speed
            ),
            channel
        );
    }
    if (inside(id, DIAMOND_LIGHTBARS_4_START, DIAMOND_LIGHTBARS_4_END, DIAMOND_LIGHTBARS_5_START, DIAMOND_LIGHTBARS_5_END)) {
        uint channel = id % 18;
        id /= 18;

        float[12] dimmers;
        for (int i = 0; i < 12; i++) {
            dimmers[i] = clamp(bass * 6.0 + 0.5 - abs(float(i) - 5.5), 0.0, 1.0);
        }
        return decode(
            TiltingLightbar(
                0.25, // tilt
                rainbow_chrono(float(id) / 40.0), // color
                clamp(10.0 * (bass - 1.0) + 1.0, 0.0, 1.0), // strobe
                dimmers // dimmers
            ),
            channel
        );
    }
    if (inside(id, CEILING_SCREEN_POSITIONERS_START, CEILING_SCREEN_POSITIONERS_END)) {
        uint channel = id % 7;
        id /= 7;
        return decode(
            XYRotationPositioner(
                false, // take control
                0.5, // x
                0.5, // y
                0.5 // rotation
            ),
            channel
        );
    }
    if (inside(id, DIAMOND_SCREEN_POSITIONERS_START, DIAMOND_SCREEN_POSITIONERS_END)) {
        uint channel = id % 7;
        id /= 7;
        return decode(
            XYRotationPositioner(
                false, // take control
                0.5, // x
                0.5, // y
                0.5 // rotation
            ),
            channel
        );
    }
    if (inside(id, TRUSS_POSITIONERS_5_START, TRUSS_POSITIONERS_5_END, TRUSS_POSITIONERS_6_START, TRUSS_POSITIONERS_6_END)) {
        uint channel = id % 2;
        id /= 2;
        return decode(
            VerticalPositioner(
                false, // take control
                0.5 // position
            ),
            channel
        );
    }
    return 0;
}

void main() {
    ivec2 coords = ivec2(UV * vec2(HNODE_RESOLUTION));
    uint channel;
    uint bit;
    bool is_checksum;
    get_hnode_channel(coords, channel, bit, is_checksum);
    uint value;
    if (is_checksum) {
        uint[HNODE_CHANNELS_PER_COL] vals;
        for (int i = 0; i < HNODE_CHANNELS_PER_COL; i++) {
            vals[i] = to_255(get_channel_value(channel + i));
        }
        value = checksum(vals);
    } else {
        value = to_255(get_channel_value(channel));
    }
    float val = get_bit(value, bit) ? 1.0 : 0.0 + smooth_magnitude(0.0) * 0.001 + rainbow_chrono(0.5).x * 0.001;
    COLOR = vec4(val, val, val, 1.0);
}
