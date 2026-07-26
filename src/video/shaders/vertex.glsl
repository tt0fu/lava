#version 460

#include "lib/push_constants.glsl"

VKO_DECLARE_STORAGE_BUFFER(transform, Transform {
    mat3 mat;
})

#define transform vko_buffer(transform, transform_buffer_id)

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 UV;

void main() {
    UV = uv;
    gl_Position = vec4((transform.mat * vec3(position, 1.0)).xy, depth, 1.0);
}
