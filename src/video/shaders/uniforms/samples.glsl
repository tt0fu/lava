#ifndef SAMPLES
#define SAMPLES

#include "../lib/consts.glsl"

layout(set = 0, binding = 2) uniform Samples {
    uint samples_start;
    float samples_data[SAMPLE_COUNT];
};

#endif
