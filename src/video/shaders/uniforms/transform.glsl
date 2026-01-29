#ifndef SCALE_X
#define SCALE_X

#include "../lib/consts.glsl"

layout(set = 0, binding = 0) uniform Transform {
    mat3 transform;
};

#endif
