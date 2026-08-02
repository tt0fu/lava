use std::sync::Arc;

use vulkano::shader::{ShaderModule, SpecializedShaderModule};

vulkano_shaders::shader! {
    lang: "glsl",
    shaders: {
        vertex: {
            ty: "vertex",
            path: "shaders/vertex.glsl",
        },
        dft: {
            ty: "compute",
            path: "shaders/compute/dft.glsl",
        },
        analysis: {
            ty: "compute",
            path: "shaders/compute/analysis.glsl",
        },
        simple: {
            ty: "fragment",
            path: "shaders/simple.glsl",
        },
        clock: {
            ty: "fragment",
            path: "shaders/clock.glsl",
        },
        waveform: {
            ty: "fragment",
            path: "shaders/waveform.glsl",
        },
        spectrogram: {
            ty: "fragment",
            path: "shaders/spectrogram.glsl",
        },
        bands: {
            ty: "fragment",
            path: "shaders/bands.glsl",
        },
    },
}

pub fn specialize(module: &Arc<ShaderModule>) -> Arc<SpecializedShaderModule> {
    module.specialize(&[(0, 48000u32.into())])
}
