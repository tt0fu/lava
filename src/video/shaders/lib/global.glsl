#ifndef LIB_GLOBAL
#define LIB_GLOBAL

#include <vulkano.glsl>

#include "consts.glsl"

VKO_DECLARE_STORAGE_BUFFER(global_buffer, GlobalParams {
    float time;
    float delta;
})

#define GLOBAL vko_buffer(global_buffer, global_buffer_id)

#endif
