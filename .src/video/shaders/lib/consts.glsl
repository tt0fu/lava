#ifndef CONSTS
#define CONSTS

layout(constant_id = 0) const uint SAMPLE_COUNT = 8192u;
layout(constant_id = 1) const uint BIN_COUNT = 256u;
layout(constant_id = 2) const uint SAMPLE_RATE = 48000u;

float SAMPLE_COUNT_F = float(SAMPLE_COUNT);
float BIN_COUNT_F = float(BIN_COUNT);
float SAMPLE_RATE_F = float(SAMPLE_RATE);

float LOWEST_FREQUENCY = SAMPLE_RATE_F / SAMPLE_COUNT_F;
float EXP_BINS = floor(BIN_COUNT_F / log2(SAMPLE_RATE / (2.0 * LOWEST_FREQUENCY)));

#endif
