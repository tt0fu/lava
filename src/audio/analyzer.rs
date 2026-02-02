use crate::audio::Stream;

use super::CircularBuffer;
use glam::Vec2;
use std::f32::consts::PI;

struct Consts {
    lowest_freq: f32,
    exp_bins: f32,
}

#[derive(Clone)]
struct BinData {
    window_start: usize,
    window_weights: Vec<f32>,
    complex_exponentials: Vec<Vec2>,
    total_window: f32,
}

#[derive(Clone)]
pub struct AnalysisData {
    pub dft: Vec<Vec2>,
    pub period: f32,
    pub focus: f32,
    pub center_sample: f32,
    pub bass: f32,
    pub chrono: f32,
}

pub struct Analyzer {
    buffer_size: usize,
    bin_count: usize,
    sample_rate: u32,

    lowest_frequency: f32,
    exp_bins: f32,

    buffer: CircularBuffer<f32>,
    dft_lut: Vec<BinData>,

    gain: f32,
    since_last_analysis: u64,

    focus: f32,
    chrono: u64,
    analysis_data: Option<AnalysisData>,
}

impl Analyzer {
    fn window(x: f32) -> f32 {
        if x < -1.0 || x > 1.0 {
            0.0
        } else {
            const A: f32 = 10.0;
            (A * (1.0 - x * x).max(0.0).sqrt()).exp() * (-A).exp()
        }
    }

    fn bin(exp_bins: f32, lowest_frequency: f32, frequency: f32) -> f32 {
        exp_bins * (frequency / lowest_frequency).log2()
    }

    fn frequency(exp_bins: f32, lowest_frequency: f32, bin: f32) -> f32 {
        lowest_frequency * (bin / exp_bins).exp2()
    }

    pub fn new(buffer_size: usize, bin_count: usize, sample_rate: u32) -> Self {
        let mut dft_lut = vec![
            BinData {
                window_start: 0,
                window_weights: Vec::new(),
                complex_exponentials: Vec::new(),
                total_window: 0.0,
            };
            bin_count
        ];

        let buffer_size_f = buffer_size as f32;
        let bin_count_f = bin_count as f32;
        let sample_rate_f = sample_rate as f32;

        let lowest_frequency = sample_rate_f / buffer_size_f;
        let exp_bins = (bin_count_f / (buffer_size_f / 2.0).log2()).floor();

        for bin in 0..bin_count {
            let frequency = Self::frequency(exp_bins, lowest_frequency, bin as f32);
            let sample_period = sample_rate_f / frequency;
            let phase_delta = PI * 2.0 / sample_period;
            let window_size = (8.0 * sample_period).min(buffer_size_f);
            let window_start_f = ((buffer_size_f - window_size) * 0.5).floor();
            let window_end_f = ((buffer_size_f + window_size) * 0.5).ceil();

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

            dft_lut[bin] = BinData {
                window_start,
                window_weights,
                complex_exponentials,
                total_window,
            };
        }

        Self {
            buffer_size,
            bin_count,
            sample_rate,
            lowest_frequency,
            exp_bins,
            buffer: CircularBuffer::new(buffer_size, 0.0),
            dft_lut,
            gain: 1.0,
            since_last_analysis: 0,
            focus: 0.5,
            chrono: 0,
            analysis_data: None,
        }
    }

    fn get_bin(&self, frequency: f32) -> f32 {
        Self::bin(self.exp_bins, self.lowest_frequency, frequency)
    }

    fn get_frequency(&self, bin: f32) -> f32 {
        Self::frequency(self.exp_bins, self.lowest_frequency, bin)
    }

    pub fn push(&mut self, new_sample: &f32) {
        self.gain += 0.00001;
        let volume = (new_sample * self.gain).abs();
        if volume > 1.0 {
            self.gain /= volume;
        }
        self.buffer.push(&(new_sample * self.gain));
        self.since_last_analysis += 1;

        self.analysis_data = None;
    }

    pub fn update(&mut self, stream: &mut Stream) {
        let new_samples = stream.get_samples();
        for sample in &new_samples {
            self.push(sample);
        }
    }

    pub fn get_buffer(&self) -> &CircularBuffer<f32> {
        &self.buffer
    }

    fn get_bass_eq(&self, bin: f32) -> f32 {
        let frequency = self.get_frequency(bin);
        (1.0 - frequency / 200.0).max(0.0)
    }

    pub fn analyze(&mut self) -> AnalysisData {
        match &self.analysis_data {
            Some(info) => info.clone(),
            None => {
                let buffer_size_f = self.buffer_size as f32;
                let bin_count_f = self.bin_count as f32;
                let sample_rate_f = self.sample_rate as f32;

                let mut dft = vec![Vec2::new(0.0, 0.0); self.bin_count];

                let mut mx = 0.0;
                let mut max_bin = 1;
                let mut cur;
                let mut prev = 0.0;
                let mut prevprev = 0.0;

                let mut bass_total = self.get_bass_eq(0.0);
                let mut bass_sum = bass_total * prev;

                for bin in 0..self.bin_count {
                    let bin_f = bin as f32;
                    
                    let bin_data = &self.dft_lut[bin];
                    let mut amplitude = Vec2::new(0.0, 0.0);

                    for i in 0..bin_data.window_weights.len() {
                        let sample_index = bin_data.window_start + i;
                        let mult = self.buffer[sample_index] * bin_data.window_weights[i];
                        amplitude += bin_data.complex_exponentials[i] * mult;
                    }
                    dft[bin] = amplitude / bin_data.total_window;
                    cur = dft[bin].length();

                    let bass_eq = self.get_bass_eq(bin_f);
                    bass_sum += bass_eq * cur;
                    bass_total += bass_eq;

                    if (prev >= cur)
                        && (prev >= prevprev)
                        && (prev * (1.0 - (bin_f) / (bin_count_f)) > mx)
                    {
                        mx = prev;
                        max_bin = bin - 1;
                    }

                    prevprev = prev;
                    prev = cur;
                }

                let bass = (bass_sum / bass_total * 10.0).clamp(0.0, 1.0);
                self.chrono += ((self.since_last_analysis as f32) * bass) as u64;
                self.since_last_analysis = 0;

                let frequency = self.get_frequency(max_bin as f32);
                let period = sample_rate_f / frequency;
                let phase = dft[max_bin];
                let angle = (phase.y.atan2(phase.x)) / (PI * 2.0) - 0.25;
                let center_sample =
                    (angle + (buffer_size_f * self.focus / period).ceil()) * period;

                let ans = AnalysisData {
                    dft,
                    period,
                    focus: self.focus,
                    center_sample,
                    bass,
                    chrono: (self.chrono as f32) / sample_rate_f,
                };

                self.analysis_data = Some(ans.clone());
                ans
            }
        }
    }
}
