use std::{path::PathBuf, sync::Arc};

use crate::{audio::AudioEngine, config::Config, stats::FrameTimer, video::VideoEngine};

use anyhow::Result;
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

    window: Option<Arc<Window>>,

    frame_timer: FrameTimer,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>, config: &Config, config_path: &PathBuf) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            audio_engine: AudioEngine::new(config)?,
            video_engine: VideoEngine::new(&event_loop, config, config_path)?,
            window: None,
            frame_timer: FrameTimer::new(),
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(&self.config.window_title)
                        .with_decorations(self.config.window_decorations)
                        .with_resizable(false)
                        .with_inner_size(self.config.window_size),
                )
                .expect("Failed to create window"),
        );

        self.window = Some(window.clone());
        self.video_engine
            .init(&window, &self.config)
            .expect("Failed to initialize the video engine");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
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
            WindowEvent::Resized(_) => {
                self.video_engine.resize().expect("Failed to resize window");
            }
            WindowEvent::RedrawRequested => {
                if self.config.frame_times {
                    self.frame_timer.start_frame();
                }

                self.video_engine
                    .redraw(
                        &self
                            .window
                            .as_mut()
                            .expect("Fetching a size of a window that hasn't been created yet")
                            .inner_size(),
                        &self.audio_engine.update(),
                    )
                    .expect("Failed to redraw the image");

                if self.config.frame_times {
                    self.frame_timer.end_frame();
                }
                self.window
                    .as_ref()
                    .expect("Requesting a redraw on a window that hasn't been created yet")
                    .request_redraw();
            }
            _ => {}
        }
    }
}
