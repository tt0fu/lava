use super::{RenderContext, RenderEngine};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

pub struct App {
    render_engine: RenderEngine,
    render_context: Option<RenderContext>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        Self {
            render_engine: RenderEngine::new(event_loop),
            render_context: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.render_context = Some(RenderContext::new(&self.render_engine, event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let render_context = self.render_context.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                render_context.recreate_swapchain = true;
            }
            WindowEvent::RedrawRequested => {
                render_context.redraw(&self.render_engine);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let render_context = self.render_context.as_mut().unwrap();
        render_context.window.request_redraw();
    }
}
