use nannou::geom::Rect;
use nannou::geom::Vec2;
use nannou::rand::prelude::*;

use std::f32::consts::PI;

const PI2: f32 = PI * 2.0;

/// Returns a random unit vector.
pub fn random_dir(rand: &mut SmallRng) -> Vec2 {
    let t = rand.gen_range(0.0..PI2);
    Vec2::new(t.cos(), t.sin())
}

/// Returns a random point in the [rect].
pub fn random_point_in_rect(rand: &mut SmallRng, rect: Rect) -> Vec2 {
    Vec2::new(rect.x.lerp(rand.gen()), rect.y.lerp(rand.gen()))
}
