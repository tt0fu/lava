#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, ColorParams {
    vec4 color;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    COLOR = MATERIAL.color;
}
