#ifndef GRIDNODE
#define GRIDNODE
const ivec2 GRIDNODE_RESOLUTION = ivec2(120, 13);
const ivec2 HNODE_RESOLUTION = ivec2(480, 52);
const int HNODE_CHANNELS_PER_COL = 6;
const int HNODE_CRC_BITS = 4;

float split_fine(float value, uint channel) {
    return channel == 0u ? value : fract(value * 255.0);
}

float split_color(vec3 color, uint channel) {
    return channel == 0u ? color.r : channel == 1u ? color.g : color.b;
}

uint channel_id(vec2 uv, ivec2 resolution) {
    ivec2 coords = ivec2(uv * vec2(resolution));
    return uint(coords.x * resolution.y + coords.y);
}

#define FROM_DMX(universe, id) ((universe - 1) * 512 + (id - 1))

uint to_255(float value) {
    return clamp(uint(value * 255.0), 0, 255);
}

bool get_bit(uint value, uint bit) {
    return ((value >> bit) & 1u) == 1u;
}

// for given integer coordinates of the hnode square,
// give the channel number, the bit index and if the square is a checksum square
void get_hnode_channel(ivec2 coords, out uint channel, out uint bit, out bool is_checksum) {
    is_checksum = coords.y >= (HNODE_CHANNELS_PER_COL * 8);
    channel = coords.x * HNODE_CHANNELS_PER_COL + (is_checksum ? 0 : coords.y / 8);
    bit = 7 - (coords.y % 8);
}

uint checksum(uint[HNODE_CHANNELS_PER_COL] vals) {
    uint crc = 0u;
    uint polynomial = 0x03u;

    for (uint i = 0; i < HNODE_CHANNELS_PER_COL; i++) {
        uint v = vals[i];
        for (int bit = 7; bit >= 0; --bit) {
            uint inBit = (v >> bit) & 1u;
            bool top = (crc & 0x8u) != 0u;
            crc = ((crc << 1) | inBit) & 0xFu;
            if (top) crc ^= polynomial;
        }
    }
    return (crc << HNODE_CRC_BITS) & 0xFFu;
}

struct Mover {
    float pan;
    float tilt;
    float zoom;
    float dimmer;
    float strobe;
    vec3 color;
    float gobo_speed;
    int gobo; // [0,15]
    float speed;
};

float decode(Mover mover, uint channel) {
    switch (channel) {
        case 0:
        return split_fine(mover.pan, channel);
        case 1:
        return split_fine(mover.pan, channel);
        case 2:
        return split_fine(mover.tilt, channel - 2u);
        case 3:
        return split_fine(mover.tilt, channel - 2u);
        case 4:
        return mover.zoom;
        case 5:
        return mover.dimmer;
        case 6:
        return mover.strobe;
        case 7:
        return split_color(mover.color, channel - 7u);
        case 8:
        return split_color(mover.color, channel - 7u);
        case 9:
        return split_color(mover.color, channel - 7u);
        case 10:
        return mover.gobo_speed;
        case 11:
        return float(mover.gobo) / 15.0;
        case 12:
        return mover.speed;
    }
    return 0.0;
}

struct Laser {
    float pan;
    float tilt;
    float len;
    float width;
    float flatness;
    float beam_count;
    float spin_speed;
    vec3 color;
    float dimmer;
    float beam_thickness;
    float speed;
};

float decode(Laser laser, uint channel) {
    switch (channel) {
        case 0:
        return laser.pan;
        case 1:
        return laser.tilt;
        case 2:
        return laser.len;
        case 3:
        return laser.width;
        case 4:
        return laser.flatness;
        case 5:
        return laser.beam_count;
        case 6:
        return laser.spin_speed;
        case 7:
        return split_color(laser.color, channel - 7u);
        case 8:
        return split_color(laser.color, channel - 7u);
        case 9:
        return split_color(laser.color, channel - 7u);
        case 10:
        return laser.dimmer;
        case 11:
        return laser.beam_thickness;
        case 12:
        return laser.speed;
    }
    return 0.0;
}

struct SimpleLight {
    float dimmer;
    vec3 color;
    float strobe;
};

float decode(SimpleLight simpleLight, uint channel) {
    switch (channel) {
        case 0:
        return simpleLight.dimmer;
        case 1:
        return split_color(simpleLight.color, channel - 1u);
        case 2:
        return split_color(simpleLight.color, channel - 1u);
        case 3:
        return split_color(simpleLight.color, channel - 1u);
        case 4:
        return simpleLight.strobe;
    }
    return 0.0;
}

struct TripleRopeRig {
    bool take_control;
    float rope_1;
    float rope_2;
    float rope_3;
};

float decode(TripleRopeRig tripleRopeRig, uint channel) {
    switch (channel) {
        case 0:
        return tripleRopeRig.take_control ? 1.0 : 0.0;
        case 1:
        return tripleRopeRig.rope_1;
        case 2:
        return tripleRopeRig.rope_2;
        case 3:
        return tripleRopeRig.rope_3;
    }
    return 0.0;
}

struct QuadRopeRig {
    bool take_control;
    float rope_1;
    float rope_2;
    float rope_3;
    float rope_4;
};

float decode(QuadRopeRig quadRopeRig, uint channel) {
    switch (channel) {
        case 0:
        return quadRopeRig.take_control ? 1.0 : 0.0;
        case 1:
        return quadRopeRig.rope_1;
        case 2:
        return quadRopeRig.rope_2;
        case 3:
        return quadRopeRig.rope_3;
        case 4:
        return quadRopeRig.rope_4;
    }
    return 0.0;
}

struct XYRotationPositioner {
    bool take_control;
    float x;
    float y;
    float rotation;
};

float decode(XYRotationPositioner xyRotationPositioner, uint channel) {
    switch (channel) {
        case 0:
        return xyRotationPositioner.take_control ? 1.0 : 0.0;
        case 1:
        return split_fine(xyRotationPositioner.x, channel - 1u);
        case 2:
        return split_fine(xyRotationPositioner.x, channel - 1u);
        case 3:
        return split_fine(xyRotationPositioner.y, channel - 3u);
        case 4:
        return split_fine(xyRotationPositioner.y, channel - 3u);
        case 5:
        return split_fine(xyRotationPositioner.rotation, channel - 5u);
        case 6:
        return split_fine(xyRotationPositioner.rotation, channel - 5u);
    }
    return 0.0;
}

struct VerticalPositioner {
    bool take_control;
    float vertical_position;
};

float decode(VerticalPositioner verticalPositioner, uint channel) {
    switch (channel) {
        case 0:
        return verticalPositioner.take_control ? 1.0 : 0.0;
        case 1:
        return verticalPositioner.vertical_position;
    }
    return 0.0;
}

struct TiltingLightbar {
    float tilt;
    vec3 color;
    float strobe;
    float[12] dimmers;
};

float decode(TiltingLightbar tiltingLightbar, uint channel) {
    switch (channel) {
        case 0:
        return split_fine(tiltingLightbar.tilt, channel);
        case 1:
        return split_fine(tiltingLightbar.tilt, channel);
        case 2:
        return split_color(tiltingLightbar.color, channel - 2u);
        case 3:
        return split_color(tiltingLightbar.color, channel - 2u);
        case 4:
        return split_color(tiltingLightbar.color, channel - 2u);
        case 5:
        return tiltingLightbar.strobe;
    }
    if (channel < 18) {
        return tiltingLightbar.dimmers[channel - 6u];
    }
    return 0.0;
}

struct AudiolinkThemeColorControl {
    bool enable;
    vec3 color1;
    vec3 color2;
    vec3 color3;
    vec3 color4;
};

float decode(AudiolinkThemeColorControl audiolinkThemeColorControl, uint channel) {
    if (channel == 0u) {
        return audiolinkThemeColorControl.enable ? 1.0 : 0.0;
    }
    switch ((channel - 1u) / 3u) {
        case 0:
        return split_color(audiolinkThemeColorControl.color1, channel - 1u);
        case 1:
        return split_color(audiolinkThemeColorControl.color1, channel - 4u);
        case 2:
        return split_color(audiolinkThemeColorControl.color1, channel - 7u);
        case 3:
        return split_color(audiolinkThemeColorControl.color1, channel - 10u);
    }
    return 0.0;
}

#endif
