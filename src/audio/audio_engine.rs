use crate::{
    audio::{Analyzer, AudioData, Stream},
    config::Config,
};

pub struct AudioEngine {
    stream: Stream,
    analyzer: Analyzer,
}

impl AudioEngine {
    pub fn new(config: &Config) -> Self {
        Self {
            stream: Stream::new(
                config.sample_rate,
                config.channels,
                config.fetch_buffer_size,
                config.store_buffer_size,
            ),
            analyzer: Analyzer::new(config.sample_count, config.bin_count, config.sample_rate),
        }
    }

    pub fn update(&mut self) -> AudioData {
        let new_samples = self.stream.get_samples();
        for sample in &new_samples {
            self.analyzer.push(sample);
        }
        self.analyzer.analyze()
    }
}
