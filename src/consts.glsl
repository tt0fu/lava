#ifndef CONSTS
#define CONSTS

const uint SAMPLE_COUNT = 8192u; // must be equal to the value in config.rs
const uint BIN_COUNT = 256u; // must be equal to the value in config.rs
const float SAMPLE_RATE = 48000.0; // must be equal to the value in config.rs

const float SAMPLE_COUNT_F = float(SAMPLE_COUNT);
const float BIN_COUNT_F = float(BIN_COUNT);
const float LOWEST_FREQUENCY = SAMPLE_RATE / SAMPLE_COUNT_F;
const float EXP_BINS = floor(BIN_COUNT_F / log2(SAMPLE_RATE / (2.0 * LOWEST_FREQUENCY)));

#endif
