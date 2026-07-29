#version 460

#include "lib/push_constants.glsl"
#include "lib/transform.glsl"

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 UV;

void main() {
    UV = uv;
    gl_Position = vec4((TRANSFORM.mat * vec3(position, 1.0)).xy, panel_depth, 1.0);
}
