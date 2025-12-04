use crate::circular_buffer::CircularBuffer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct SampleBuffer {
    buffer: Arc<Mutex<CircularBuffer<f32>>>,
    _stream: cpal::Stream,
}

impl SampleBuffer {
    pub fn new(size: usize) -> Self {
        let device = cpal::default_host()
            .default_input_device()
            .expect("No input device available");

        let config: cpal::StreamConfig = device
            .default_input_config()
            .expect("No default input config available")
            .into();

        let buffer = Arc::new(Mutex::new(CircularBuffer::new(size, 0.0)));
        let buffer_clone = buffer.clone();

        let channels = config.channels as usize;

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mut buf = buffer_clone.lock().expect("failed to lock buffer mutex");
                    for frame in data.chunks(channels) {
                        buf.push(frame.iter().sum::<f32>() / (frame.len() as f32));
                    }
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
            .expect("failed to build stream");

        stream.play().expect("error playing stream");

        Self {
            buffer,
            _stream: stream,
        }
    }

    pub fn get_samples(&mut self) -> Vec<f32> {
        let mut out = Vec::new();
        let mut buf = self.buffer.lock().expect("failed to lock buffer mutex");

        while let Some(sample) = buf.pop() {
            out.push(sample);
        }
        out
    }
}
