use lava::video::app::App;
use std::error::Error;
use winit::event_loop::EventLoop;

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(&event_loop);

    event_loop.run_app(&mut app)
}
