use super::CircularBuffer;
use glam::Vec2;
use std::f32::consts::PI;

struct Consts {
    lowest_freq: f32,
    exp_bins: f32,
}

#[derive(Clone)]
pub struct StabilizationInfo {
    pub period: f32,
    pub focus: f32,
    pub center_sample: f32,
}

struct BinData {
    window_start: usize,
    window_weights: Vec<f32>,
    complex_exponentials: Vec<Vec2>,
    total_window: f32,
}

pub struct Analyzer<const BUFFER_SIZE: usize, const BIN_COUNT: usize, const SAMPLE_RATE: u32> {
    consts: Consts,

    buffer: CircularBuffer<f32, BUFFER_SIZE>,
    gain: f32,

    bin_data: [BinData; BIN_COUNT],
    dft: Option<[Vec2; BIN_COUNT]>,

    focus: f32,
    stabilization_info: Option<StabilizationInfo>,
}

impl<const BUFFER_SIZE: usize, const BIN_COUNT: usize, const SAMPLE_RATE: u32>
    Analyzer<BUFFER_SIZE, BIN_COUNT, SAMPLE_RATE>
{
    fn window(x: f32) -> f32 {
        if x < -1.0 || x > 1.0 {
            0.0
        } else {
            const A: f32 = 2.0;
            (A * (1.0 - x * x).max(0.0).sqrt()).exp() * (-A).exp()
        }
    }

    pub fn new() -> Self {
        let mut bin_data = [(); BIN_COUNT].map(|_| BinData {
            window_start: 0,
            window_weights: Vec::new(),
            complex_exponentials: Vec::new(),
            total_window: 0.0,
        });

        let buffer_size_f = BUFFER_SIZE as f32;
        let sample_rate_f = SAMPLE_RATE as f32;

        let lowest_freq = sample_rate_f / buffer_size_f;
        let exp_bins = (BIN_COUNT as f32 / (buffer_size_f / 2.0).log2()).floor();

        for bin in 0..BIN_COUNT {
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
                let cur_window =
                    Self::window((sample_index as f32 * 2.0 - buffer_size_f) / window_size);
                window_weights.push(cur_window);
                total_window += cur_window;

                let phase = initial_phase + phase_delta * i as f32;
                complex_exponentials.push(Vec2::new(phase.cos(), phase.sin()));
            }

            bin_data[bin] = BinData {
                window_start,
                window_weights,
                complex_exponentials,
                total_window,
            };
        }

        Self {
            consts: Consts {
                lowest_freq,
                exp_bins,
            },
            buffer: CircularBuffer::new(0.0),
            gain: 1.0,
            bin_data,
            dft: None,
            focus: 0.5,
            stabilization_info: None,
        }
    }

    pub fn push(&mut self, new_sample: &f32) {
        self.gain += 0.00001;
        let volume = (new_sample * self.gain).abs();
        if volume > 0.9 {
            self.gain /= volume / 0.9;
        }
        if (new_sample * self.gain).abs() > 1.0 {
            self.gain /= new_sample.abs();
        }
        self.buffer.push(&(new_sample * self.gain));

        self.dft = None;
        self.stabilization_info = None;
    }

    pub fn get_buffer(&self) -> CircularBuffer<f32, BUFFER_SIZE> {
        self.buffer.clone()
    }

    fn compute_ft(&mut self) {
        if self.dft == None {
            let mut dft = [Vec2::new(0.0, 0.0); BIN_COUNT];
            for bin in 0..BIN_COUNT {
                let bin_data = &self.bin_data[bin];
                let mut amplitude = Vec2::new(0.0, 0.0);

                for i in 0..bin_data.window_weights.len() {
                    let sample_index = bin_data.window_start + i;
                    let mult = self.buffer[sample_index] * bin_data.window_weights[i];
                    amplitude += bin_data.complex_exponentials[i] * mult;
                }
                dft[bin] = amplitude / bin_data.total_window;
            }
            self.dft = Some(dft);
        }
    }

    fn compute_stabilization_info(&mut self) {
        self.compute_ft();
        let dft = self.dft.unwrap();
        let mut mx = 0.0;
        let mut max_bin = 1;
        let mut prev = dft[0].length();
        let mut cur = dft[1].length();
        let mut next = dft[2].length();
        for i in 1..(BIN_COUNT - 3) {
            if (cur >= prev)
                && (cur >= next)
                && (cur * (1.0 - (i as f32) / (BIN_COUNT as f32)) > mx)
            {
                mx = cur;
                max_bin = i;
            }

            prev = cur;
            cur = next;
            next = dft[i + 3].length();
        }
        let frequency =
            (2.0 as f32).powf(max_bin as f32 / self.consts.exp_bins) * self.consts.lowest_freq;

        let period = SAMPLE_RATE as f32 / frequency;
        let phase = dft[max_bin];
        let angle = (phase.y.atan2(phase.x)) / (PI * 2.0) - 0.25;
        let center_sample = (angle + (BUFFER_SIZE as f32 * self.focus / period).ceil()) * period;

        self.stabilization_info = Some(StabilizationInfo {
            period,
            focus: self.focus,
            center_sample,
        })
    }

    pub fn get_ft(&mut self) -> [Vec2; BIN_COUNT] {
        self.compute_ft();
        self.dft.unwrap()
    }

    pub fn get_stabilization_info(&mut self) -> StabilizationInfo {
        self.compute_stabilization_info();
        self.stabilization_info.clone().unwrap()
    }
}
