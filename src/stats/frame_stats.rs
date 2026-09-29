use std::time::Duration;

pub struct FrameStats {
    pub frame_count: usize,
    pub average: Duration,
    pub min: Duration,
    pub max: Duration,
    pub p90: Duration,
    pub p99: Duration,
    pub p999: Duration,
}

impl FrameStats {
    pub fn print(&self) {
        println!(
            "{} frames: \n  avg: {:?} ({:.1} fps)\n  min: {:?} ({:.1} fps)\n  max: {:?} ({:.1} fps)\n  90%: {:?} ({:.1} fps)\n  99%: {:?} ({:.1} fps)\n  99.9%: {:?} ({:.1} fps)",
            self.frame_count,
            self.average,
            fps(self.average),
            self.min,
            fps(self.min),
            self.max,
            fps(self.max),
            self.p90,
            fps(self.p90),
            self.p99,
            fps(self.p99),
            self.p999,
            fps(self.p999)
        );
    }
}

fn fps(frame_time: Duration) -> f64 {
    if frame_time > Duration::ZERO {
        Duration::from_secs(1).div_duration_f64(frame_time)
    } else {
        0.0
    }
}
