use vulkano::{buffer::Buffer, memory::allocator::DeviceLayout};
use vulkano_taskgraph::{Id, TaskContext};

use crate::video::{
    parameters::{Layout, WriteMut},
    shaders,
};

pub struct Waveform {
    start: usize,
    gain: f32,
    focus: f32,
    samples: Vec<f32>,

    max_samples: usize,
    unwritten_samples: usize,
}

impl Waveform {
    pub fn new(max_samples: usize) -> Self {
        Self {
            start: 0,
            gain: 1.0,
            focus: 0.5,
            samples: Vec::with_capacity(max_samples),
            max_samples,
            unwritten_samples: 0,
        }
    }

    pub fn push(&mut self, sample: f32) {
        self.gain += 0.000001;
        let volume = (sample * self.gain).abs();
        if volume > 1.0 {
            self.gain /= volume;
        }

        if self.samples.len() < self.max_samples {
            self.samples.push(sample * self.gain);
        } else {
            self.samples[self.start] = sample * self.gain;
            self.start = (self.start + 1) % self.max_samples;
        }
        self.unwritten_samples = (self.unwritten_samples + 1).max(self.max_samples);
    }

    pub fn push_slice(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.push(sample);
        }
    }
}

impl Layout for Waveform {
    fn layout(&self) -> DeviceLayout {
        DeviceLayout::new_unsized::<shaders::Waveform>(self.max_samples as u64).unwrap()
    }
}
impl WriteMut for Waveform {
    fn write(&mut self, id: Id<Buffer>, tcx: &mut TaskContext<'_>) {
        let guard = tcx.write_buffer::<shaders::Waveform>(id, ..);
        guard.sample_count = self.samples.len() as u32;
        guard.start = self.start as u32;
        guard.focus = self.focus;

        guard.period = 100.0;
        guard.center_sample = self.samples.len() as f32 / 2.0;

        if self.unwritten_samples == 0 {
            return;
        }

        let len = self.samples.len();

        if self.unwritten_samples >= self.max_samples {
            guard.samples[..len].copy_from_slice(&self.samples);
            guard.start = self.start as u32;
        } else {
            if len < self.max_samples {
                let old_len = len - self.unwritten_samples;
                guard.samples[old_len..len].copy_from_slice(&self.samples[old_len..len]);
                guard.start = 0;
            } else {
                let phys_start =
                    (self.start + self.max_samples - self.unwritten_samples) % self.max_samples;
                let first_part = self.unwritten_samples.min(self.max_samples - phys_start);

                guard.samples[phys_start..phys_start + first_part]
                    .copy_from_slice(&self.samples[phys_start..phys_start + first_part]);

                if first_part < self.unwritten_samples {
                    let second_part = self.unwritten_samples - first_part;
                    guard.samples[0..second_part].copy_from_slice(&self.samples[0..second_part]);
                }
                guard.start = self.start as u32;
            }
        }

        self.unwritten_samples = 0;
    }
}
