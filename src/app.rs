use std::sync::Arc;

use crate::{audio::AudioEngine, config::Config, stats::FrameTimer, video::VideoEngine};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

pub struct App {
    config: Config,

    audio_engine: AudioEngine,
    video_engine: VideoEngine,

    window: Option<Arc<Box<dyn Window>>>,

    frame_timer: FrameTimer,
}

impl App {
    pub fn new(event_loop: &EventLoop, config: &Config) -> Self {
        Self {
            config: config.clone(),
            audio_engine: AudioEngine::new(config),
            video_engine: VideoEngine::new(event_loop),
            window: None,
            frame_timer: FrameTimer::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("lava visualizer")
                        .with_decorations(false)
                        .with_resizable(false)
                        .with_surface_size(self.config.window_size),
                )
                .unwrap(),
        );

        self.window = Some(window.clone());
        self.video_engine.init(&window, &self.config);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if self.config.frame_times {
                    self.frame_timer.print_results();
                }
                event_loop.exit();
            }
            WindowEvent::SurfaceResized(_) => {
                self.video_engine.resize();
            }
            WindowEvent::RedrawRequested => {
                if self.config.frame_times {
                    self.frame_timer.start_frame();
                }

                self.video_engine.redraw(
                    &self.window.as_mut().unwrap().surface_size(),
                    &self.audio_engine.update(),
                );

                if self.config.frame_times {
                    self.frame_timer.end_frame();
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}
