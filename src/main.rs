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
    let config = {
        let args = std::env::args().collect::<Vec<String>>();
        match args.len() {
            1 => Config::default(),
            2 => Config::from_jsonc(args[1].as_str()),
            _ => {
                panic!("Usage: lava [path/to/config.jsonc]");
            }
        }
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let app = App::new(&event_loop, &config);

    event_loop.run_app(app)
}
