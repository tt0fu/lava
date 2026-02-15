mod app;
mod audio;
mod config;
mod stats;
mod video;

use app::App;
use std::error::Error;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::config::Config;

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let app = App::new(&event_loop, &Config::grey_venue());

    event_loop.run_app(app)
}
