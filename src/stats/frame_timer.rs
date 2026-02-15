use std::time::{Duration, Instant};

pub struct FrameTimer {
    start_time: Instant,
    frame_start: Instant,
    frame_times: Vec<Duration>,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            frame_start: Instant::now(),
            frame_times: Vec::new(),
        }
    }

    pub fn start_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    pub fn end_frame(&mut self) {
        self.frame_times
            .push(Instant::now().duration_since(self.frame_start));
    }

    pub fn results(&self) -> (usize, Duration, Duration, Duration) {
        let len = self.frame_times.len();
        let sum = self.frame_times.iter().sum::<Duration>();
        let avg = sum.div_f64(len as f64);
        let min = self.frame_times.iter().min().unwrap().clone();
        let max = self.frame_times.iter().max().unwrap().clone();
        (len, avg, min, max)
    }

    pub fn print_results(&self) {
        let (len, avg, min, max) = self.results();
        println!(
            "{} frames: avg={:?} ({} fps), min={:?}, max={:?}",
            len,
            avg,
            Duration::from_secs(1).div_duration_f64(avg),
            min,
            max,
        );
    }

    pub fn clear_frame_times(&mut self) {
        self.frame_times.clear();
    }
}
