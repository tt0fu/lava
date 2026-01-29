vulkano_shaders::shader! {
    shaders: {
        vertex: {
            ty: "vertex",
            path: "src/video/shaders/vertex.glsl",
        },
        waveform: {
            ty: "fragment",
            path: "src/video/shaders/waveform.glsl",
        },
        rainbow: {
            ty: "fragment",
            path: "src/video/shaders/rainbow.glsl",
        },
        spectrogram: {
            ty: "fragment",
            path: "src/video/shaders/spectrogram.glsl",
        },
        gray_venue_gridnode: {
            ty: "fragment",
            path: "src/video/shaders/gray_venue_gridnode.glsl",
        },
        image: {
            ty: "fragment",
            path: "src/video/shaders/image.glsl",
        }
    }
}

impl WaveformParameters {
    pub const DEFAULT: Self = Self {
        line_width: 50.0,
        gain: 0.9,
        lightness: 0.8,
        chroma: 0.1,
        scale: 1.0,
        pattern_speed: 1.0,
        scroll_speed: 2.0,
    };
}

impl Default for WaveformParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl RainbowParameters {
    pub const DEFAULT: Self = Self {
        lightness: 0.8,
        chroma: 0.1,
        scale: 1.0,
        pattern_speed: 1.0,
        scroll_speed: 2.0,
    };
}

impl Default for RainbowParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl SpectrogramParameters {
    pub const DEFAULT: Self = Self { gain: 1.5 };
}

impl Default for SpectrogramParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl GrayVenueGridnodeParameters {
    pub const DEFAULT: Self = Self {
        lightness: 0.8,
        chroma: 0.1,
    };
}

impl Default for GrayVenueGridnodeParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ImageParameters {
    pub const DEFAULT: Self = Self {
        alpha_cutoff: 0.5,
        scale_min: 0.5,
        scale_max: 1.0,
    };
}

impl Default for ImageParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}
