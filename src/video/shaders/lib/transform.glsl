#ifndef LIB_TRANSFORM
#define LIB_TRANSFORM

#include <vulkano.glsl>

#include "consts.glsl"

VKO_DECLARE_STORAGE_BUFFER(transform_buffer, Transform {
    mat3 mat;
    float aspect_ratio;
})

#define TRANSFORM vko_buffer(transform_buffer, transform_buffer_id)

#endif
