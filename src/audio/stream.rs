use crate::audio::circular_buffer::CircularBuffer;

use cpal::{
    SampleRate,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::{Arc, Mutex};

pub struct Stream<const BUFFER_SIZE: usize, const CHANNELS : usize> {
    buffer: Arc<Mutex<CircularBuffer<[f32; CHANNELS], BUFFER_SIZE>>>,
    _stream: cpal::Stream,
}

impl<const BUFFER_SIZE: usize, const CHANNELS : usize> Stream<BUFFER_SIZE, CHANNELS> {
    pub fn new(sample_rate: u32, buffer_size: u32) -> Self {
        let device = cpal::default_host()
            .default_input_device()
            .expect("No input device available");


        let config = cpal::StreamConfig {
            channels: CHANNELS as u16,
            sample_rate: SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(buffer_size),
        };
        let buffer = Arc::new(Mutex::new(CircularBuffer::new([0.0; CHANNELS])));
        let buffer_clone = buffer.clone();


        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mut buf = buffer_clone.lock().expect("failed to lock buffer mutex");
                    for frame in data.chunks(CHANNELS) {
                        let mut static_frame = [0.0; CHANNELS];
                        for i in 0..CHANNELS {
                            static_frame[i] = frame[i];
                        }
                        buf.push(static_frame);
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

    pub fn get_samples(&mut self) -> Vec<[f32; CHANNELS]> {
        let mut out = Vec::new();
        let mut buf = self.buffer.lock().expect("failed to lock buffer mutex");

        while let Some(sample) = buf.pop() {
            out.push(sample);
        }
        out
    }
}
