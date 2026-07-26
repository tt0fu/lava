use lava::video::{app::App, audio_settings::AudioSettings};
use std::error::Error;
use winit::event_loop::EventLoop;

fn main() -> Result<(), impl Error> {
    let audio_settings = AudioSettings {
        sample_rate: 48000,
        channel_count: 1,
        sample_count: 8192,
        bin_count: 1024,
    };

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(&event_loop, audio_settings);

    event_loop.run_app(&mut app)
}
