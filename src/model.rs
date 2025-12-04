use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Position {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

pub const POSITIONS: [Position; 4] = [
    Position {
        position: [-0.5, -0.5],
    },
    Position {
        position: [-0.5, 0.5],
    },
    Position {
        position: [0.5, -0.5],
    },
    Position {
        position: [0.5, 0.5],
    },
];

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Uv {
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
}

pub const UVS: [Uv; 4] = [
    Uv {
        uv: [0.0, 0.0],
    },
    Uv {
        uv: [0.0, 1.0],
    },
    Uv {
        uv: [1.0, 0.0],
    },
    Uv {
        uv: [1.0, 1.0],
    },
];

pub const INDICES: [u16; 6] = [
    0, 1, 2, 1, 2, 3
];
