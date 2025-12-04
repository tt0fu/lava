#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    f_color = vec4(v_uv, worldview[1][0], 1.0);
}
