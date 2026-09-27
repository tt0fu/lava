pub struct AudioSettings {
    pub sample_rate: u32,
    pub channel_count: u16,
    pub stream_buffer_size: u32,
    pub sample_count: usize,
    pub dft_bin_count: usize,
    pub bands_history_length: usize,
}
