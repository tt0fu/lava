use super::CircularBuffer;

use anyhow::Result;
use cpal::{
    BufferSize::Fixed,
    ErrorKind, SampleFormat, StreamConfig, SupportedBufferSize, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::{Arc, Mutex};

pub struct Stream {
    buffer: Arc<Mutex<CircularBuffer<f32>>>,
    _stream: cpal::Stream,
}

impl Stream {
    pub fn new(
        sample_rate: u32,
        channels: u16,
        fetch_buffer_size: u32,
        store_buffer_size: usize,
    ) -> Result<Self> {
        let device = default_host()
            .default_input_device()
            .expect("No audio input devices available");

        println!("Using audio device: {}", device);

        let config = StreamConfig {
            channels,
            sample_rate,
            buffer_size: Fixed(fetch_buffer_size * channels as u32),
        };
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(store_buffer_size, 0.0)));
        let buffer_clone = buffer.clone();

        let stream = {
            let res = device.build_input_stream(
                config,
                move |data: &[f32], _| {
                    let mut buf = buffer_clone
                        .lock()
                        .expect("Failed to lock audio buffer mutex");
                    for frame in data.chunks(channels as usize) {
                        buf.push(&(frame.iter().sum::<f32>() / frame.len() as f32));
                    }
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            );
            match res {
                Ok(stream) => stream,
                Err(err) => {
                    if err.kind() == ErrorKind::UnsupportedConfig {
                        eprintln!(
                            "Unsupported audio stream config: channels={}, fetch_buffer_size={}, sample_rate={}.",
                            channels, fetch_buffer_size, sample_rate
                        );
                        eprintln!("Supported audio stream configs:");
                        for config in device
                            .supported_input_configs()?
                            .filter(|config| config.sample_format() == SampleFormat::F32)
                        {
                            eprintln!(
                                "channels={}, fetch_buffer_size={}, sample_rate=[{}, {}]",
                                config.channels(),
                                match config.buffer_size() {
                                    SupportedBufferSize::Range { min, max } =>
                                        format!("[{}, {}]", min, max),
                                    SupportedBufferSize::Unknown => "Unknown".to_string(),
                                },
                                config.min_sample_rate(),
                                config.max_sample_rate()
                            );
                        }
                    }
                    panic!("Failed to build audio stream: {}", err)
                }
            }
        };

        stream.play().expect("Error playing audio stream");

        Ok(Self {
            buffer,
            _stream: stream,
        })
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
