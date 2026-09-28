use anyhow::Result;
use lava::{config::Config, video::app::App};
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("usage: lava <path/to/config.jsonc>");
    let config = Config::load(path)?;

    let debug = cfg!(debug_assertions) || cfg!(feature = "debug");
    let event_loop = EventLoop::new()?;
    let mut app = App::new(&event_loop, config, debug)?;

    event_loop.run_app(&mut app)?;
    Ok(())
}
