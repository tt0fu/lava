use glam::Mat2;

struct Shader {}

trait Parameter {}

impl Parameter for f32 {}

struct Material {
    shader: Shader,
    paremeters: Vec<Box<dyn Parameter>>,
}

struct Panel {
    order: u32,
    is_static: bool,
    transform: Mat2,
    material: Material,
}

struct Scene {
    panels: Vec<Panel>,
}

struct App {
    scenes: Vec<Scene>,
}
