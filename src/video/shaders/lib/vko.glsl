#ifndef LIB_VKO
#define LIB_VKO

#include <vulkano.glsl>

// vulkano.glsl declares the bindless storage buffers as writable. The vertex and fragment
// stages only ever read them, but without the `vertexPipelineStoresAndAtomics` and
// `fragmentStoresAndAtomics` device features enabled the validation layer complains about
// this. Marking the buffers `readonly` in those stages fixes the warnings.
#if defined(GL_VERTEX_SHADER) || defined(GL_FRAGMENT_SHADER)
#undef VKO_DECLARE_STORAGE_BUFFER
#define VKO_DECLARE_STORAGE_BUFFER(NAME, BLOCK)                                                    \
    layout(set = VKO_GLOBAL_SET, binding = VKO_STORAGE_BUFFER_BINDING)                             \
        readonly buffer BLOCK _vko_##NAME##_storage_buffers[];
#endif

#endif
