use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    device::{Device, Queue},
    instance::{Instance, debug::DebugUtilsMessenger},
};
use vulkano_taskgraph::{
    Id,
    resource::{Flight, Resources, ResourcesCreateInfo},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::{
    audio::{audio_settings::AudioSettings, stream::Stream},
    config::Config,
    stats::frame_timer::FrameTimer,
    video::{
        MAX_FRAMES_IN_FLIGHT, debug::create_debug_messenger, device::create_device,
        instance::create_instance, render_context::RenderContext, scene_data::SceneData,
    },
};

pub struct App {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub resources: Arc<Resources>,
    pub flight_id: Id<Flight>,

    pub scene_data: Arc<SceneData>,
    pub audio_settings: Arc<AudioSettings>,

    pub render_context: Option<RenderContext>,
    pub stream: Arc<Stream>,
    pub frame_timer: FrameTimer,

    /// Kept alive for as long as the app runs so validation messages keep being reported.
    _debug_messenger: Option<DebugUtilsMessenger>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>, config: Config, debug: bool) -> Result<Self> {
        let instance = create_instance(event_loop, debug);
        let debug_messenger = if debug {
            create_debug_messenger(&instance)
        } else {
            None
        };
        let (device, queue) = create_device(&instance, event_loop);
        let scene_data = SceneData::new(&device, &config)?;
        let audio_settings = Arc::new(config.audio);

        let resources = Resources::new(
            &device,
            &ResourcesCreateInfo {
                bindless_context: Some(&Default::default()),
                ..Default::default()
            },
        )
        .unwrap();
        let flight_id = resources.create_flight(MAX_FRAMES_IN_FLIGHT).unwrap();
        let stream = Arc::new(Stream::new(&audio_settings).unwrap());

        Ok(App {
            instance,
            device,
            queue,
            resources,
            flight_id,
            scene_data: Arc::new(scene_data),
            audio_settings,
            render_context: None,
            stream,
            frame_timer: FrameTimer::new(),
            _debug_messenger: debug_messenger,
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.render_context = Some(RenderContext::new(
            event_loop,
            &self.instance,
            &self.device,
            &self.queue,
            &self.resources,
            self.flight_id,
            &self.scene_data,
            &self.audio_settings,
            &self.stream,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let rcx = self.render_context.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                self.frame_timer.print_results();
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                rcx.window_state.recreate_requested = true;
            }
            WindowEvent::RedrawRequested => {
                self.frame_timer.start_frame();
                rcx.redraw(&self.resources, self.flight_id);
                self.frame_timer.end_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let rcx = self.render_context.as_mut().unwrap();
        rcx.window_state.window.request_redraw();
    }
}
