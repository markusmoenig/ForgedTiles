use crate::prelude::*;

#[inline(always)]
pub fn sdf_box2d(p: Vec2f, width: f32, height: f32) -> f32 {
    let d = abs(p) - vec2f(width, height);
    length(max(d, Vec2f::zero())) + min(max(d.x, d.y), 0.0)
}
