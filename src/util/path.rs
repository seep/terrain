use nannou::glam::*;

pub fn smooth_path(points: &[Vec2]) -> SmoothPathIterator {
    SmoothPathIterator { points, index: 0 }
}

pub struct SmoothPathIterator<'a> {
    points: &'a [Vec2],
    index: usize,
}

impl Iterator for SmoothPathIterator<'_> {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;

        self.index += 1;

        if index == 0 {
            return Some(self.points[index]);
        }

        if index == self.points.len() - 1 {
            return Some(self.points[index]);
        }

        if index < self.points.len() {
            let prev = self.points[index - 1];
            let next = self.points[index + 1];
            let midd = Vec2::lerp(prev, next, 0.5);

            let p = self.points[index].lerp(midd, 0.25);

            return Some(p);
        }

        None
    }
}
