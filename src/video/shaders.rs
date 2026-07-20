vulkano_shaders::shader! {
    lang: "glsl",
    root_path_env: "CARGO_MANIFEST_DIR",
    shaders: {
        vertex: {
            ty: "vertex",
            path: "src/video/shaders/vertex.glsl",
        },
        fragment: {
            ty: "fragment",
            path: "src/video/shaders/fragment.glsl",
        },
    },
}
