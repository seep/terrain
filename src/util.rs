use nannou::geom::*;
use nannou::math::*;

pub mod path;
pub use path::*;

pub mod poisson;
pub use poisson::*;

pub mod voronoi;
pub use voronoi::*;

pub mod priority_index;
pub use priority_index::*;

#[allow(dead_code)]
pub fn saturate(n: f32) -> f32 {
    n.clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn expand_rect(rect: Rect, margin: f32) -> Rect {
    Rect::from_xy_wh(rect.xy(), rect.wh() + 2.0 * margin)
}

pub fn max_position(arr: &[f32]) -> Option<usize> {
    if arr.is_empty() {
        return None;
    }

    let mut max_value = arr[0];
    let mut max_index = 0;

    for (i, e) in arr.iter().enumerate() {
        if max_value < *e {
            max_value = *e;
            max_index = i;
        }
    }

    Some(max_index)
}

/// Returns the min and max values of an f32 slice.
#[allow(dead_code)]
pub fn minmax(arr: &[f32]) -> Option<(f32, f32)> {
    if arr.is_empty() {
        return None;
    }

    let mut min = arr[0];
    let mut max = arr[0];

    for e in arr.iter() {
        min = e.min(min);
        max = e.max(max);
    }

    Some((min, max))
}

/// Normalize a slice of f32 into the range \[0.0, 1.0\] using the min and max elements.
pub fn normalize(arr: &mut [f32]) {
    if let Some((min, max)) = minmax(arr) {
        for e in arr.iter_mut() {
            *e = map_range(*e, min, max, 0.0, 1.0);
        }
    }
}

#[allow(dead_code)]
pub fn map_clamp(val: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    map_range(val, in_min, in_max, out_min, out_max).clamp(out_min, out_max)
}

#[allow(dead_code)]
pub fn lerp(val: f32, min: f32, max: f32) -> f32 {
    map_range(val, 0.0, 1.0, min, max)
}

#[allow(dead_code)]
pub fn unlerp(val: f32, min: f32, max: f32) -> f32 {
    map_range(val, min, max, 0.0, 1.0)
}

/// Find the mean of [values] for a subset of [indices].
pub fn indexed_mean(values: &[f32], indices: &[usize]) -> f32 {
    if indices.is_empty() {
        return 0.0;
    }

    let mut sum = 0.0;

    for i in indices.iter() {
        sum += values[*i];
    }

    sum / indices.len() as f32
}
