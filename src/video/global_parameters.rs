use std::time::{Duration, Instant};

use crate::video::{parameters::TypedParameters, shaders};

pub struct GlobalParameters {
    pub start_time: Instant,
    pub previous: Instant,

    pub time: Duration,
    pub delta: Duration,
}

impl GlobalParameters {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            previous: now,
            time: Duration::ZERO,
            delta: Duration::ZERO,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta = now - self.previous;
        self.time = now - self.start_time;
        self.previous = now;
    }
}

impl TypedParameters for GlobalParameters {
    type Content = shaders::GlobalParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            time: self.time.as_secs_f32(),
            delta: self.delta.as_secs_f32(),
        }
    }
}
