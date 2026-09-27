use anyhow::Result;
use cpal::{
    BufferSize::Fixed,
    ErrorKind, SampleFormat, StreamConfig, SupportedBufferSize, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use vulkano::buffer::Buffer;
use vulkano_taskgraph::{Id, TaskContext};

use std::sync::{Arc, Mutex};

use crate::{
    audio::{audio_settings::AudioSettings, waveform::Waveform},
    video::parameters::{Layout, Write, WriteMut},
};

pub struct Stream {
    waveform: Arc<Mutex<Waveform>>,
    _stream: cpal::Stream,
}

impl Stream {
    pub fn new(audio_settings: &AudioSettings) -> Result<Self> {
        let device = default_host()
            .default_input_device()
            .expect("No audio input devices available");

        println!("Using audio device: {}", device);

        let config = StreamConfig {
            channels: audio_settings.channel_count,
            sample_rate: audio_settings.sample_rate,
            buffer_size: Fixed(
                audio_settings.stream_buffer_size * audio_settings.channel_count as u32,
            ),
        };
        let buffer = Arc::new(Mutex::new(Waveform::new(&audio_settings)));
        let buffer_clone = buffer.clone();
        let channel_count = audio_settings.channel_count;

        let stream = {
            let res = device.build_input_stream(
                config,
                move |data: &[f32], _| {
                    let mut buf = buffer_clone
                        .lock()
                        .expect("Failed to lock audio waveform mutex");
                    buf.push_slice(
                        data.chunks(channel_count as usize)
                            .map(|f| f.iter().sum::<f32>() / f.len() as f32)
                            .collect::<Vec<f32>>()
                            .as_slice(),
                    );
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            );
            match res {
                Ok(stream) => stream,
                Err(err) => {
                    if err.kind() == ErrorKind::UnsupportedConfig {
                        eprintln!(
                            "Unsupported audio stream config: channel_count={}, stream_buffer_size={}, sample_rate={}.",
                            audio_settings.channel_count,
                            audio_settings.stream_buffer_size,
                            audio_settings.sample_rate
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
            waveform: buffer,
            _stream: stream,
        })
    }
}

impl Layout for Stream {
    fn layout(&self) -> vulkano::memory::allocator::DeviceLayout {
        let waveform = self.waveform.lock().expect("failed to lock waveform mutex");
        waveform.layout()
    }
}

impl Write for Stream {
    fn write(&self, id: Id<Buffer>, tcx: &mut TaskContext<'_>) {
        let mut waveform = self.waveform.lock().expect("failed to lock waveform mutex");
        waveform.write(id, tcx);
    }
}
