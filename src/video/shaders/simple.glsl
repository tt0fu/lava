#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material_buffer, SimpleParams {
    float value;
})

#define MATERIAL vko_buffer(material_buffer, material_buffer_id)

void main() {
    COLOR = vec4(UV, MATERIAL.value, 1.0);
}
