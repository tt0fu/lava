#ifndef STABILIZATION
#define STABILIZATION

#include "../lib/consts.glsl"

layout(set = 0, binding = 3) uniform Stabilization {
    float period;
    float focus;
    float center_sample;
};

#endif
