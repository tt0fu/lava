use std::time::{Duration, Instant};

use crate::stats::frame_stats::FrameStats;

pub struct FrameTimer {
    frame_start: Instant,
    frame_times: Vec<Duration>,
    sorted_cache: Option<Vec<Duration>>,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            frame_start: Instant::now(),
            frame_times: Vec::new(),
            sorted_cache: None,
        }
    }

    pub fn start_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    pub fn end_frame(&mut self) {
        self.frame_times
            .push(Instant::now().duration_since(self.frame_start));
        self.sorted_cache = None;
    }

    pub fn print_results(&mut self) {
        self.stats().print();
    }

    fn stats(&mut self) -> FrameStats {
        let frame_count = self.frame_times.len();
        let sum = self.frame_times.iter().sum::<Duration>();
        FrameStats {
            frame_count,
            average: if frame_count > 0 {
                sum.div_f64(frame_count as f64)
            } else {
                Duration::ZERO
            },
            min: self.min(),
            max: self.max(),
            p90: self.percentile(0.90),
            p99: self.percentile(0.99),
            p999: self.percentile(0.999),
        }
    }

    fn min(&self) -> Duration {
        self.frame_times
            .iter()
            .min()
            .copied()
            .unwrap_or(Duration::ZERO)
    }

    fn max(&self) -> Duration {
        self.frame_times
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::ZERO)
    }

    fn percentile(&mut self, p: f64) -> Duration {
        let len = self.frame_times.len();
        if len == 0 {
            return Duration::ZERO;
        }

        if self.sorted_cache.is_none() {
            let mut sorted = self.frame_times.clone();
            sorted.sort();
            self.sorted_cache = Some(sorted);
        }
        let sorted = self.sorted_cache.as_ref().unwrap();

        let index = ((p * (len - 1) as f64).round() as usize).min(len - 1);
        sorted[index]
    }
}
