use vulkano::padded::Padded;

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
        simple_pattern: {
            ty: "fragment",
            path: "src/video/shaders/simple_pattern.glsl",
        },
        masked_pattern: {
            ty: "fragment",
            path: "src/video/shaders/masked_pattern.glsl",
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

impl Pattern {
    pub const DEFAULT: Self = Self {
        color: [1.0, 1.0, 1.0],
        use_rainbow: 1,
        lightness: 0.8,
        chroma: 0.1,
        scale: 1.0,
        repeats: 2.0,
        pattern_speed: 1.0,
        scroll_speed: 2.0,
    };
}

impl WaveformParameters {
    pub const DEFAULT: Self = Self {
        pattern: Padded(Pattern::DEFAULT),
        line_width: 50.0,
        gain: 0.9,
    };
}

impl Default for WaveformParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl SimplePatternParameters {
    pub const DEFAULT: Self = Self {
        pattern: Pattern::DEFAULT,
    };
}

impl SpectrogramParameters {
    pub const DEFAULT: Self = Self {
        pattern: Padded(Pattern::DEFAULT),
        gain: 1.5,
    };
}

impl GrayVenueGridnodeParameters {
    pub const DEFAULT: Self = Self {
        lightness: 0.8,
        chroma: 0.1,
    };
}

impl ImageParameters {
    pub const DEFAULT: Self = Self {
        scale_min: 0.5,
        scale_max: 1.0,
    };
}

impl MaskedPatternParameters {
    pub const DEFAULT: Self = Self {
        pattern: Padded(Pattern::DEFAULT),
        scale_min: 0.5,
        scale_max: 1.0,
    };
}
