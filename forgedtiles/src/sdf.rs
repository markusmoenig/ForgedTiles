use crate::prelude::*;

/// 2D hash, taken from https://www.shadertoy.com/view/4djSRW
#[inline(always)]
pub fn hash21(p: Vec2f) -> f32 {
    let mut p3 = frac(vec3f(p.x * 0.1031, p.y * 0.1031, p.x * 0.1031));
    let dot = dot(p3, vec3f(p3.y + 33.333, p3.z + 33.333, p3.x + 33.333));

    p3.x += dot;
    p3.y += dot;
    p3.z += dot;
    ((p3.x + p3.y) * p3.z).fract()
}

#[inline(always)]
pub fn rot(a: f32) -> Mat2f {
    Mat2f::new(a.cos(), -a.sin(), a.sin(), a.cos())
}

#[inline(always)]
pub fn sdf_box2d(p: Vec2f, width: f32, height: f32, r: f32) -> f32 {
    let d = abs(p) - vec2f(width, height) + vec2f(r, r);
    length(max(d, Vec2f::zero())) + min(max(d.x, d.y), 0.0) - r
}
