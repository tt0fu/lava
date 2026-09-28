use glam::{Mat3, Vec2, Vec4};
use vulkano::pipeline::graphics::color_blend::AttachmentBlend;
use vulkano_taskgraph::descriptor_set::SampledImageId;

use crate::video::{
    scene_data::{Element, SceneData},
    shaders,
    transform::{aabb, to_ndc, unit_to_local},
};

/// The element tree resolved against a concrete screen size.
pub struct ResolvedScene {
    pub groups: Vec<ResolvedGroup>,
    pub panels: Vec<ResolvedPanel>,
    /// Index of the screen group.
    pub root: usize,
}

pub struct ResolvedGroup {
    pub parent: Option<usize>,
    /// Direct children, in draw order (back to front).
    pub children: Vec<ResolvedChild>,
    pub background: Vec4,
    pub blend: AttachmentBlend,
    pub order: u32,
    /// Maps group-local pixel coordinates `[0, size]` to absolute screen pixels.
    pub local_to_abs: Mat3,
    pub size: Vec2,
    pub aabb_origin: Vec2,
    pub aabb_size: Vec2,
    /// Placement of this group's AABB quad in the parent target's NDC (unit quad -> NDC).
    pub composite_ndc: Mat3,
    /// Maps the composite quad's UV `[0, 1]²` to this group's local pixel coordinates.
    pub uv_to_local: Mat3,
}

pub enum ResolvedChild {
    Panel(usize),
    Group(usize),
}

impl ResolvedGroup {
    /// The synthetic panel's `GroupParams` for a given sampled target image.
    pub fn params(&self, image: SampledImageId) -> shaders::GroupParams {
        shaders::GroupParams {
            image: image.into(),
            uv_to_local: [
                self.uv_to_local.x_axis.to_array().into(),
                self.uv_to_local.y_axis.to_array().into(),
                self.uv_to_local.z_axis.to_array().into(),
            ],
            size: self.size.to_array().into(),
        }
    }
}

pub struct ResolvedPanel {
    pub material: usize,
    pub blend: AttachmentBlend,
    /// Maps the unit quad to the containing group target's NDC.
    pub ndc: Mat3,
    pub aspect_ratio: f32,
}

impl ResolvedScene {
    pub fn new(scene: &SceneData, screen_size: Vec2) -> Self {
        let mut resolved = Self {
            groups: Vec::new(),
            panels: Vec::new(),
            root: 0,
        };
        // The screen is an implicit group whose target is the swapchain.
        let root = resolved.add_group(
            None,
            screen_size,
            Mat3::IDENTITY,
            scene.background_color,
            AttachmentBlend::alpha(),
            0,
        );
        resolved.add_element(root, &scene.root, screen_size, Mat3::IDENTITY);
        resolved
    }

    fn add_group(
        &mut self,
        parent: Option<usize>,
        size: Vec2,
        local_to_abs: Mat3,
        background: Vec4,
        blend: AttachmentBlend,
        order: u32,
    ) -> usize {
        let corners = [
            local_to_abs.transform_point2(Vec2::ZERO),
            local_to_abs.transform_point2(Vec2::new(size.x, 0.0)),
            local_to_abs.transform_point2(Vec2::new(0.0, size.y)),
            local_to_abs.transform_point2(size),
        ];
        let (aabb_origin, aabb_size) = aabb(&corners);

        // `abs_from_uv` maps the composite quad's UV [0, 1]² to absolute screen pixels.
        let abs_from_uv = Mat3::from_translation(aabb_origin) * Mat3::from_scale(aabb_size);
        let (composite_ndc, uv_to_local) = match parent {
            Some(parent) => {
                let parent = &self.groups[parent];
                (
                    to_ndc(parent.aabb_origin, parent.aabb_size)
                        * abs_from_uv
                        * unit_to_local(Vec2::ONE),
                    local_to_abs.inverse() * abs_from_uv,
                )
            }
            None => (Mat3::IDENTITY, Mat3::IDENTITY),
        };

        let index = self.groups.len();
        self.groups.push(ResolvedGroup {
            parent,
            children: Vec::new(),
            background,
            blend,
            order,
            local_to_abs,
            size,
            aabb_origin,
            aabb_size,
            composite_ndc,
            uv_to_local,
        });
        index
    }

    fn add_element(
        &mut self,
        parent: usize,
        element: &Element,
        parent_size: Vec2,
        parent_local_to_abs: Mat3,
    ) {
        match element {
            Element::Panel(panel) => {
                let abs = parent_local_to_abs * panel.transform.matrix_px(parent_size);
                let parent_group = &self.groups[parent];
                let index = self.panels.len();
                self.panels.push(ResolvedPanel {
                    material: panel.material,
                    blend: panel.blend.clone(),
                    ndc: to_ndc(parent_group.aabb_origin, parent_group.aabb_size) * abs,
                    aspect_ratio: panel.transform.aspect_ratio(parent_size),
                });
                self.groups[parent]
                    .children
                    .push(ResolvedChild::Panel(index));
            }
            Element::Group(group) => {
                let abs = parent_local_to_abs * group.transform.matrix_px(parent_size);
                let size = group.transform.size(parent_size);
                let local_to_abs = abs * unit_to_local(size).inverse();
                let index = self.add_group(
                    Some(parent),
                    size,
                    local_to_abs,
                    group.background,
                    group.blend.clone(),
                    group.order,
                );
                self.groups[parent]
                    .children
                    .push(ResolvedChild::Group(index));

                let mut children = group.children.iter().collect::<Vec<_>>();
                children.sort_by_key(|element| match element {
                    Element::Panel(panel) => panel.order,
                    Element::Group(group) => group.order,
                });
                for child in children {
                    self.add_element(index, child, size, local_to_abs);
                }
            }
        }
    }
}
