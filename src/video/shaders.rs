vulkano_shaders::shader! {
    lang: "glsl",
    root_path_env: "CARGO_MANIFEST_DIR",
    shaders: {
        vertex: {
            ty: "vertex",
            path: "src/video/shaders/vertex.glsl",
        },
        dft: {
            ty: "compute",
            path: "src/video/shaders/compute/dft.glsl",
        },
        analysis: {
            ty: "compute",
            path: "src/video/shaders/compute/analysis.glsl",
        },
        simple: {
            ty: "fragment",
            path: "src/video/shaders/simple.glsl",
        },
        clock: {
            ty: "fragment",
            path: "src/video/shaders/clock.glsl",
        },
        waveform: {
            ty: "fragment",
            path: "src/video/shaders/waveform.glsl",
        },
        spectrogram: {
            ty: "fragment",
            path: "src/video/shaders/spectrogram.glsl",
        },
        bands: {
            ty: "fragment",
            path: "src/video/shaders/bands.glsl",
        },
    },
}
