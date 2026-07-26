#version 460

#include "lib/push_constants.glsl"
#include "lib/in_out.glsl"

VKO_DECLARE_STORAGE_BUFFER(material, SimpleParams {
    float value;
})

#define material vko_buffer(material, material_buffer_id)

void main() {
    COLOR = vec4(UV, material.value, 1.0);
}
