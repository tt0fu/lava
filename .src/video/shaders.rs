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
        neon_symphony_hnode: {
            ty: "fragment",
            path: "src/video/shaders/neon_symphony_hnode.glsl",
        },
        image: {
            ty: "fragment",
            path: "src/video/shaders/image.glsl",
        }
    },
}
