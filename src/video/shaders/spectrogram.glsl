#version 450

#include "../../consts.glsl"

layout(location = 0) in vec2 UV;
layout(location = 0) out vec4 COLOR;

#include "uniforms/dft.glsl"

layout(set = 0, binding = 10) uniform SpectrogramParameters {
    float gain;
};

void main() {
    float val = step(1.0 - UV.y, smooth_magnitude(UV.x * (BIN_COUNT - 1)) * gain);
    if (val < 0.5) {
        discard;
    }
    COLOR = vec4(val, val, val, val);
}
