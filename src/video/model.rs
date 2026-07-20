use vulkano_macros::{BufferContents, Vertex};

#[derive(Clone, Copy, BufferContents, Vertex)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
}

pub const VERTICES: [MyVertex; 4] = [
    MyVertex {
        position: [-1.0, -1.0],
        uv: [0.0, 0.0],
    },
    MyVertex {
        position: [-1.0, 1.0],
        uv: [0.0, 1.0],
    },
    MyVertex {
        position: [1.0, -1.0],
        uv: [1.0, 0.0],
    },
    MyVertex {
        position: [1.0, 1.0],
        uv: [1.0, 1.0],
    },
];
