#ifndef GRIDNODE
#define GRIDNODE
const ivec2 RESOLUTION = ivec2(120, 13);

float split_fine(float value, uint channel) {
    return channel == 0u ? value : fract(value * 256.0);
}

float split_color(vec3 color, uint channel) {
    return channel == 0u ? color.r : channel == 1u ? color.g : color.b;
}

uint channel_id(vec2 uv) {
    ivec2 coords = ivec2(uv * vec2(RESOLUTION));
    return uint(coords.x * RESOLUTION.y + coords.y);
}

struct Mover {
    float pan;
    float tilt;
    float zoom;
    float dimmer;
    float strobe;
    vec3 color;
    float gobo;
    float gobo_speed;
    float speed;
};

float decode(Mover mover, uint channel) {
    if (channel == 0u || channel == 1u) {
        return split_fine(mover.pan, channel);
    }
    if (channel == 2u || channel == 3u) {
        return split_fine(mover.tilt, channel - 2u);
    }
    if (channel == 4u) {
        return mover.zoom;
    }
    if (channel == 5u) {
        return mover.dimmer;
    }
    if (channel == 6u) {
        return mover.strobe;
    }
    if (7u <= channel && channel <= 9u) {
        return split_color(mover.color, channel - 7u);
    }
    if (channel == 10u) {
        return mover.gobo_speed;
    }
    if (channel == 11u) {
        return mover.gobo;
    }
    if (channel == 12u) {
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
    if (channel == 0u) {
        return laser.pan;
    }
    if (channel == 1u) {
        return laser.tilt;
    }
    if (channel == 2u) {
        return laser.len;
    }
    if (channel == 3u) {
        return laser.width;
    }
    if (channel == 4u) {
        return laser.flatness;
    }
    if (channel == 5u) {
        return laser.beam_count;
    }
    if (channel == 6u) {
        return laser.spin_speed;
    }
    if (7u <= channel && channel <= 9u) {
        return split_color(laser.color, channel - 7u);
    }
    if (channel == 10u) {
        return laser.dimmer;
    }
    if (channel == 11u) {
        return laser.beam_thickness;
    }
    if (channel == 12u) {
        return laser.speed;
    }
    return 0.0;
}

struct ParLight {
    float dimmer;
    vec3 color;
    float strobe;
};

float decode(ParLight parlight, uint channel) {
    if (channel == 0u) {
        return parlight.dimmer;
    }
    if (1u <= channel && channel <= 3u) {
        return split_color(parlight.color, channel - 1u);
    }
    if (channel == 4u) {
        return parlight.strobe;
    }
    return 0.0;
}

#endif
