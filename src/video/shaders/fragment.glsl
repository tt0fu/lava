#version 460

#include "header.glsl"

layout(location = 0) in vec2 UV;
layout(location = 0) out vec4 COLOR;

void main() {
    float val = step(distance(UV, vec2(0.5, 0.5)), fract(global.time) * 0.5) * 0.5 + 0.5;
    COLOR = vec4(val * local.col, 1.0);
}
