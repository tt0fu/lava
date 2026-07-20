#ifndef IMAGE
#define IMAGE

layout(set = 0, binding = 6) uniform sampler image_sampler;
layout(set = 0, binding = 7) uniform texture2D image_texture;

vec4 image_color(vec2 uv) {
    return texture(sampler2D(image_texture, image_sampler), uv);
}

#endif
