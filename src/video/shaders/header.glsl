#ifndef PUSH_CONSTANTS
#define PUSH_CONSTANTS

#include <vulkano.glsl>

layout(push_constant) uniform PushConstants {
    StorageBufferId global_buffer_id;
    StorageBufferId transform_buffer_id;
    StorageBufferId local_buffer_id;
};

VKO_DECLARE_STORAGE_BUFFER(global, GlobalParams {
    float time;
})

#define global vko_buffer(global, global_buffer_id)


VKO_DECLARE_STORAGE_BUFFER(transform, Transform {
    mat3 mat;
})

#define transform vko_buffer(transform, transform_buffer_id)


VKO_DECLARE_STORAGE_BUFFER(local, LocalParams {
    vec3 col;
})

#define local vko_buffer(local, local_buffer_id)


#endif
