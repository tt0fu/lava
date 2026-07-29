#ifndef LIB_COMPUTE_PUSH_CONSTANTS
#define LIB_COMPUTE_PUSH_CONSTANTS

#include <vulkano.glsl>

layout(push_constant) uniform ComputePushConstants {
    StorageBufferId global_buffer_id;
    StorageBufferId waveform_buffer_id;
    StorageBufferId dft_buffer_id;
    StorageBufferId bands_buffer_id;
};

#endif
