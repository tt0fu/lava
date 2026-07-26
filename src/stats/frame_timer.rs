use std::time::{Duration, Instant};

pub struct FrameTimer {
    start_time: Instant,
    frame_start: Instant,
    frame_times: Vec<Duration>,
    sorted_cache: Option<Vec<Duration>>,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
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
        // Invalidate cache because the vector has changed
        self.sorted_cache = None;
    }

    /// Returns the value at the given percentile (0.0 .. 1.0).
    /// Uses the nearest-rank method. Panics if p is not in [0,1] or if there are no frames.
    pub fn percentile(&mut self, p: f64) -> Duration {
        assert!(
            (0.0..=1.0).contains(&p),
            "Percentile must be between 0 and 1"
        );
        let len = self.frame_times.len();
        assert!(len > 0, "No frame times recorded");

        // Ensure cache is up-to-date
        if self.sorted_cache.is_none() {
            let mut sorted = self.frame_times.clone();
            sorted.sort();
            self.sorted_cache = Some(sorted);
        }

        let sorted = self.sorted_cache.as_ref().unwrap();
        let index = ((p * (len - 1) as f64).round() as usize).min(len - 1);
        sorted[index]
    }

    /// (number of frames, average, min, max, 90%, 99%, 99.9%)
    pub fn results(
        &mut self,
    ) -> (
        usize,
        Duration,
        Duration,
        Duration,
        Duration,
        Duration,
        Duration,
    ) {
        let len = self.frame_times.len();
        let sum = self.frame_times.iter().sum::<Duration>();
        let avg = if len > 0 {
            sum.div_f64(len as f64)
        } else {
            Duration::ZERO
        };
        let min = self
            .frame_times
            .iter()
            .min()
            .copied()
            .unwrap_or(Duration::ZERO);
        let max = self
            .frame_times
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::ZERO);
        let p90 = self.percentile(0.90);
        let p99 = self.percentile(0.99);
        let p999 = self.percentile(0.999);
        (len, avg, min, max, p90, p99, p999)
    }

    fn fps(frame_time: Duration) -> f64 {
        if frame_time > Duration::ZERO {
            Duration::from_secs(1).div_duration_f64(frame_time)
        } else {
            0.0
        }
    }

    pub fn print_results(&mut self) {
        let (len, avg, min, max, p90, p99, p999) = self.results();
        println!(
            "{} frames: \n  avg: {:?} ({:.1} fps)\n  min: {:?} ({:.1} fps)\n  max: {:?} ({:.1} fps)\n  90%: {:?} ({:.1} fps)\n  99%: {:?} ({:.1} fps)\n  99.9%: {:?} ({:.1} fps)",
            len,
            avg,
            Self::fps(avg),
            min,
            Self::fps(min),
            max,
            Self::fps(max),
            p90,
            Self::fps(p90),
            p99,
            Self::fps(p99),
            p999,
            Self::fps(p999)
        );
    }

    pub fn clear_frame_times(&mut self) {
        self.frame_times.clear();
        self.sorted_cache = None;
    }
}
