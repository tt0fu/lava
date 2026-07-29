#ifndef LIB_PUSH_CONSTANTS
#define LIB_PUSH_CONSTANTS

#include <vulkano.glsl>

layout(push_constant) uniform PushConstants {
    StorageBufferId global_buffer_id;
    StorageBufferId waveform_buffer_id;
    StorageBufferId dft_buffer_id;
    StorageBufferId bands_buffer_id;
    
    StorageBufferId transform_buffer_id;
    
    StorageBufferId material_buffer_id;

    float panel_depth;
};

#endif
