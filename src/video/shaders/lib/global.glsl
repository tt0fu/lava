#ifndef GLOBAL
#define GLOBAL

#include <vulkano.glsl>

#include "consts.glsl"

VKO_DECLARE_STORAGE_BUFFER(global, GlobalParams {
    float time;
})

#define global vko_buffer(global, global_buffer_id)

#endif
