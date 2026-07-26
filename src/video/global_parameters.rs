use std::time::{Duration, Instant};
use vulkano::memory::allocator::DeviceLayout;

use crate::video::{parameters::TypedParameters, shaders};

pub struct GlobalParameters {
    pub start_time: Instant,

    pub time: Duration,
}

impl GlobalParameters {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            time: Duration::ZERO,
        }
    }

    pub fn update(&mut self) {
        self.time = Instant::now() - self.start_time;
    }

    pub fn layout() -> DeviceLayout {
        DeviceLayout::new_sized::<shaders::GlobalParams>()
    }

    pub fn get_buffer(&self) -> shaders::GlobalParams {
        shaders::GlobalParams {
            time: self.time.as_secs_f32(),
        }
    }
}

impl TypedParameters for GlobalParameters {
    type Content = shaders::GlobalParams;

    fn get_content(&self) -> Self::Content {
        Self::Content {
            time: self.time.as_secs_f32(),
        }
    }
}
