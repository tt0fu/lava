use glam::Vec4;
use vulkano::pipeline::graphics::color_blend::AttachmentBlend;

use crate::video::transform::Transform;

pub enum Element {
    Panel(Panel),
    Group(Group),
}

impl Element {
    /// The draw order among siblings (back to front).
    pub fn order(&self) -> u32 {
        match self {
            Element::Panel(panel) => panel.order,
            Element::Group(group) => group.order,
        }
    }
}

pub struct Panel {
    pub transform: Transform,
    /// Index into `SceneData::materials`.
    pub material: usize,
    pub order: u32,
    pub blend: AttachmentBlend,
}

pub struct Group {
    pub transform: Transform,
    pub background: Vec4,
    pub order: u32,
    pub blend: AttachmentBlend,
    pub children: Vec<Element>,
}
