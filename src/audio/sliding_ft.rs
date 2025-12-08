use crate::audio::circular_buffer::CircularBuffer;
use glam::Vec2;
use std::f32::consts::PI;

fn window(x: f32) -> f32 {
    if x < -1.0 || x > 1.0 {
        0.0
    } else {
        const A: f32 = 10.0;
        (A * (1.0 - x * x).max(0.0).sqrt()).exp() * (-A).exp()
    }
}

pub trait SlidingFT<const BIN_COUNT: usize> {
    fn new() -> Self;
    fn push(&mut self, new_sample: &f32);
    fn get_ft(&self) -> [Vec2; BIN_COUNT];
}

pub struct SlidingFTNaive<const BUFFER_SIZE: usize, const BIN_COUNT: usize, const SAMPLE_RATE: u32>
{
    buffer: CircularBuffer<f32, BUFFER_SIZE>,
}

impl<const BUFFER_SIZE: usize, const BIN_COUNT: usize, const SAMPLE_RATE: u32> SlidingFT<BIN_COUNT>
    for SlidingFTNaive<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>
{
    fn new() -> Self {
        Self {
            buffer: CircularBuffer::new(0.0),
        }
    }
    fn push(&mut self, new_sample: &f32) {
        self.buffer.push(&new_sample);
    }

    fn get_ft(&self) -> [Vec2; BIN_COUNT] {
        let buffer_size_f = BUFFER_SIZE as f32;
        let sample_rate_f = SAMPLE_RATE as f32;

        let lowest_freq = sample_rate_f / buffer_size_f;
        let exp_bins = (BIN_COUNT as f32 / (buffer_size_f / 2.0).log2()).floor();

        let mut bins = [Vec2::new(0.0, 0.0); BIN_COUNT];
        for bin in 0..BIN_COUNT {
            let mut amplitude = Vec2::new(0.0, 0.0);
            let frequency = lowest_freq * (bin as f32 / exp_bins).exp2();
            let sample_period = SAMPLE_RATE as f32 / frequency;
            let phase_delta = PI * 2.0 / sample_period;
            let window_size = (8.0 * sample_period).min(BUFFER_SIZE as f32);
            let window_start = ((BUFFER_SIZE as f32 - window_size) * 0.5).floor();
            let window_end = ((BUFFER_SIZE as f32 + window_size) * 0.5).ceil();
            let mut cur_phase = phase_delta * window_start;
            let mut total_window = 0.0;
            for sample_index in (window_start as usize)..(window_end as usize) {
                let cur_window =
                    window((sample_index as f32 * 2.0 - BUFFER_SIZE as f32) / window_size);
                let mult = self.buffer[sample_index] * cur_window;
                amplitude += Vec2::new(cur_phase.cos(), cur_phase.sin()) * mult;
                total_window += cur_window;
                cur_phase += phase_delta;
            }
            bins[bin] = amplitude / total_window;
        }
        bins
    }
}

// pub struct SlidingFTFast<const BUFFER_SIZE: usize, const BIN_COUNT: usize, const SAMPLE_RATE: u32>
// {
//     // todo
// }

// impl<const BUFFER_SIZE: usize, const BIN_COUNT: usize, const SAMPLE_RATE: u32> SlidingFT<BIN_COUNT>
//     for SlidingFTFast<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>
// {
//     fn new() -> Self {
//         todo!()
//     }
//     fn push(&mut self, new_sample: &f32) {
//         todo!()
//     }
//     fn get_ft(&self) -> [Vec2; BIN_COUNT] {
//         todo!()
//     }
// }

pub struct SlidingFTFast<const BUFFER_SIZE: usize, const BIN_COUNT: usize, const SAMPLE_RATE: u32> {
    buffer: CircularBuffer<f32, BUFFER_SIZE>,
    // Precomputed data for each bin
    bin_data: [BinData; BIN_COUNT],
    // Current FT values for each bin
    current_ft: [Vec2; BIN_COUNT],
}

struct BinData {
    window_start: usize,
    window_weights: Vec<f32>,
    complex_exponentials: Vec<Vec2>,
    total_window: f32,
}

impl<const BUFFER_SIZE: usize, const BIN_COUNT: usize, const SAMPLE_RATE: u32>
    SlidingFTFast<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>
{
    fn compute_bin_data(bin: usize) -> BinData {
        let buffer_size_f = BUFFER_SIZE as f32;
        let sample_rate_f = SAMPLE_RATE as f32;

        let lowest_freq = sample_rate_f / buffer_size_f;
        let exp_bins = (BIN_COUNT as f32 / (buffer_size_f / 2.0).log2()).floor();

        let frequency = lowest_freq * (bin as f32 / exp_bins).exp2();
        let sample_period = SAMPLE_RATE as f32 / frequency;
        let phase_delta = PI * 2.0 / sample_period;
        let window_size = (8.0 * sample_period).min(BUFFER_SIZE as f32);
        let window_start_f = ((BUFFER_SIZE as f32 - window_size) * 0.5).floor();
        let window_end_f = ((BUFFER_SIZE as f32 + window_size) * 0.5).ceil();

        let window_start = window_start_f as usize;
        let window_end = window_end_f as usize;
        let window_len = window_end - window_start;

        let mut window_weights = Vec::with_capacity(window_len);
        let mut complex_exponentials = Vec::with_capacity(window_len);
        let mut total_window = 0.0;

        let initial_phase = phase_delta * window_start_f;

        for i in 0..window_len {
            let sample_index = window_start + i;
            let cur_window = window((sample_index as f32 * 2.0 - buffer_size_f) / window_size);
            window_weights.push(cur_window);
            total_window += cur_window;

            let phase = initial_phase + phase_delta * i as f32;
            complex_exponentials.push(Vec2::new(phase.cos(), phase.sin()));
        }

        BinData {
            window_start,
            window_weights,
            complex_exponentials,
            total_window,
        }
    }
}

impl<const BUFFER_SIZE: usize, const BIN_COUNT: usize, const SAMPLE_RATE: u32> SlidingFT<BIN_COUNT>
    for SlidingFTFast<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>
{
    fn new() -> Self {
        let mut bin_data = [(); BIN_COUNT].map(|_| BinData {
            window_start: 0,
            window_weights: Vec::new(),
            complex_exponentials: Vec::new(),
            total_window: 0.0,
        });

        for bin in 0..BIN_COUNT {
            bin_data[bin] = Self::compute_bin_data(bin);
        }

        Self {
            buffer: CircularBuffer::new(0.0),
            bin_data,
            current_ft: [Vec2::new(0.0, 0.0); BIN_COUNT],
        }
    }

    fn push(&mut self, new_sample: &f32) {
        self.buffer.push(&new_sample);

        for bin in 0..BIN_COUNT {
            let bin_data = &self.bin_data[bin];
            let mut amplitude = Vec2::new(0.0, 0.0);

            for i in 0..bin_data.window_weights.len() {
                let sample_index = bin_data.window_start + i;
                let mult = self.buffer[sample_index] * bin_data.window_weights[i];
                amplitude += bin_data.complex_exponentials[i] * mult;
            }

            self.current_ft[bin] = amplitude / bin_data.total_window;
        }
    }

    fn get_ft(&self) -> [Vec2; BIN_COUNT] {
        self.current_ft
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, distr::Uniform};
    use std::time::Instant;

    fn check<const BIN_COUNT: usize, F: SlidingFT<BIN_COUNT>, N: SlidingFT<BIN_COUNT>>(
        mut fast: F,
        mut naive: N,
        iter_count: usize,
    ) {
        let mut rng = rand::rng();
        for iter in 0..iter_count {
            let sample = rng.sample(Uniform::new(-1.0, 1.0).unwrap());
            naive.push(&sample);
            fast.push(&sample);

            let naive_bins = naive.get_ft();
            let fast_bins = fast.get_ft();

            for i in 0..BIN_COUNT {
                assert!(
                    (naive_bins[i] - fast_bins[i]).length() < 1e-3,
                    "iter = {}, naive_bins[{}] = {}, fast_bins[{}] = {}",
                    iter,
                    i,
                    naive_bins[i],
                    i,
                    fast_bins[i]
                );
            }
        }
    }

    #[test]
    fn small() {
        const SAMPLE_RATE: u32 = 48000;
        const BUFFER_SIZE: usize = 128;
        const BIN_COUNT: usize = 16;
        let fast = SlidingFTFast::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
        let naive = SlidingFTNaive::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
        check(fast, naive, 10);
    }

    #[test]
    fn single_fill() {
        const SAMPLE_RATE: u32 = 48000;
        const BUFFER_SIZE: usize = 1024;
        const BIN_COUNT: usize = 64;
        let fast = SlidingFTFast::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
        let naive = SlidingFTNaive::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
        check(fast, naive, BUFFER_SIZE);
    }

    #[test]
    fn double_fill() {
        const SAMPLE_RATE: u32 = 48000;
        const BUFFER_SIZE: usize = 256;
        const BIN_COUNT: usize = 32;
        let fast = SlidingFTFast::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
        let naive = SlidingFTNaive::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
        check(fast, naive, BUFFER_SIZE * 2);
    }

    #[test]
    fn long() {
        const SAMPLE_RATE: u32 = 48000;
        const BUFFER_SIZE: usize = 4096;
        const BIN_COUNT: usize = 512;
        let fast = SlidingFTFast::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
        let naive = SlidingFTNaive::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
        check(fast, naive, BUFFER_SIZE * 10);
    }

    #[test]
    fn performance() {
        const SAMPLE_RATE: u32 = 48000;
        const BUFFER_SIZE: usize = 512;
        const BIN_COUNT: usize = 64;
        let mut fast = SlidingFTFast::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();
        let mut naive = SlidingFTNaive::<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>::new();

        let mut rng = rand::rng();
        let dist = Uniform::new(-1.0, 1.0).unwrap();
        let samples = (0..(BUFFER_SIZE * 3))
            .map(|_| rng.sample(dist))
            .collect::<Vec<f32>>();

        for _ in 0..10 {
            let mut sum_fast: f32 = 0.0;
            let time_start_fast = Instant::now();
            for sample in &samples {
                fast.push(&sample);
                sum_fast += fast.get_ft().iter().map(|b| b.length()).sum::<f32>();
            }
            let elapsed_fast = time_start_fast.elapsed();
            let time_fast = elapsed_fast.as_secs() as f32
                + elapsed_fast.subsec_nanos() as f32 / 1_000_000_000.0;

            let mut sum_naive: f32 = 0.0;
            let time_start_naive = Instant::now();
            for sample in &samples {
                naive.push(&sample);
                sum_naive += naive.get_ft().iter().map(|b| b.length()).sum::<f32>();
            }
            let elapsed_naive = time_start_naive.elapsed();
            let time_naive = elapsed_naive.as_secs() as f32
                + elapsed_naive.subsec_nanos() as f32 / 1_000_000_000.0;

            assert!(time_fast < time_naive, "{} {} {} {}", sum_fast, sum_naive, time_fast, time_naive);
        }
    }
}
