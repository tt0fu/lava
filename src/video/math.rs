use glam::{Mat3, Vec2};

/// Matrix mapping the unit quad `[-1, 1]^2` to a rectangle's local `[0, size]` space.
pub fn unit_to_local(size: Vec2) -> Mat3 {
    Mat3::from_scale(size)
        * Mat3::from_translation(Vec2::splat(0.5))
        * Mat3::from_scale(Vec2::splat(0.5))
}

/// Matrix mapping pixel coordinates to the normalized device coordinates of a target with the
/// given origin and size (in pixels).
pub fn to_ndc(target_origin: Vec2, target_size: Vec2) -> Mat3 {
    Mat3::from_translation(-Vec2::ONE)
        * Mat3::from_scale(2.0 / target_size)
        * Mat3::from_translation(-target_origin)
}

/// The axis-aligned bounding box of `points`, snapped to integer pixel coordinates and fully
/// covering them. Returns `(origin, size)` with `size` at least `(1, 1)`.
pub fn aabb(points: &[Vec2]) -> (Vec2, Vec2) {
    let min = points
        .iter()
        .copied()
        .fold(Vec2::splat(f32::INFINITY), |a, b| a.min(b));
    let max = points
        .iter()
        .copied()
        .fold(Vec2::splat(f32::NEG_INFINITY), |a, b| a.max(b));
    let origin = min.floor();
    let size = (max.ceil() - origin).max(Vec2::ONE);
    (origin, size)
}
