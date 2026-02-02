#ifndef TRANSFORM
#define TRANSFORM

#include "../lib/consts.glsl"

layout(set = 0, binding = 0) uniform Transform {
    mat3 transform;
};

#endif
