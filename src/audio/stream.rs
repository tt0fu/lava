use super::CircularBuffer;

use cpal::{
    BufferSize::Fixed,
    SampleRate, StreamConfig, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::{Arc, Mutex};

pub struct Stream {
    buffer: Arc<Mutex<CircularBuffer<f32>>>,
    _stream: cpal::Stream,
}

impl Stream {
    pub fn new(sample_rate: u32, fetch_buffer_size: u32, store_buffer_size: usize) -> Self {
        let device = default_host()
            .default_input_device()
            .expect("No audio input devices available");

        println!("Using audio device: {}", device.name().unwrap());

        let config = StreamConfig {
            channels: 1,
            sample_rate: sample_rate,
            buffer_size: Fixed(fetch_buffer_size),
        };
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(store_buffer_size, 0.0)));
        let buffer_clone = buffer.clone();

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mut buf = buffer_clone
                        .lock()
                        .expect("Failed to lock audio buffer mutex");
                    for frame in data {
                        buf.push(&frame);
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
        let mut result = Vec::new();
        let mut buffer = self.buffer.lock().expect("failed to lock buffer mutex");

        while let Some(sample) = buffer.pop() {
            result.push(sample);
        }
        result
    }
}
