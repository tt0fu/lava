vulkano_shaders::shader! {
    root_path_env: "CARGO_MANIFEST_DIR",
    lang: "glsl",
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
        color: {
            ty: "fragment",
            path: "src/video/shaders/color.glsl",
        },
        pattern: {
            ty: "fragment",
            path: "src/video/shaders/pattern.glsl",
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
        image: {
            ty: "fragment",
            path: "src/video/shaders/image.glsl",
        },
        group: {
            ty: "fragment",
            path: "src/video/shaders/group.glsl",
        },
        gridnode: {
            ty: "fragment",
            path: "src/video/shaders/gridnode.glsl",
        },
        hnode: {
            ty: "fragment",
            path: "src/video/shaders/hnode.glsl",
        },
    },
}
