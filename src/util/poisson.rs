use nannou::geom::*;
use nannou::math::map_range;

use nannou::rand::rngs::SmallRng;
use nannou::rand::*;

use std::f32::consts::{PI, SQRT_2};

const PI2: f32 = PI * 2.0;

/// Generate random samples within [extent] in a Poisson disk distribution, with minimum [radius] separation.
pub fn poisson(rand: &mut SmallRng, extent: Rect, radius: f32) -> Vec<Vec2> {
    let cell_size = radius / SQRT_2;
    let grid_size = (extent.wh() / cell_size).ceil().as_u32();

    let mut sampler = PoissonDiskSampler {
        cell_size,
        extent,
        radius,
        queued: vec![],
        points: vec![],
        grid: SampleGrid::new(grid_size.x as usize, grid_size.y as usize),
    };

    sampler.generate_samples(rand);
    sampler.points
}

struct PoissonDiskSampler {
    /// The range of values to generate samples in.
    extent: Rect,
    /// The min allowed radius between samples.
    radius: f32,
    /// The size of each grid cell.
    cell_size: f32,
    /// The list of point indexes to sample additional points from.
    queued: Vec<usize>,
    /// The sampled points.
    points: Vec<Vec2>,
    /// A spatial grid containing the optional index of (at most one) point in the grid cell.
    grid: SampleGrid,
}

struct SampleGrid {
    rows: usize,
    cols: usize,
    grid: Vec<Option<usize>>,
}

impl SampleGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        SampleGrid {
            cols,
            rows,
            grid: vec![None; rows * cols],
        }
    }

    pub fn get(&self, cell: (usize, usize)) -> Option<usize> {
        self.grid[cell.0 + cell.1 * self.cols]
    }

    pub fn set(&mut self, cell: (usize, usize), value: Option<usize>) {
        self.grid[cell.0 + cell.1 * self.cols] = value;
    }
}

impl PoissonDiskSampler {
    /// Attempt to generate a new point in the Poisson distribution by sampling near [from].
    fn generate_samples(&mut self, rand: &mut SmallRng) {
        let init = vec2(
            self.extent.x.lerp(rand.gen()),
            self.extent.y.lerp(rand.gen()),
        );

        self.points.push(init);
        self.queued.push(0);

        while !self.queued.is_empty() {
            let near_index = rand.gen_range(0..self.queued.len());
            let near_point = self.points[self.queued[near_index]];

            match self.generate_sample(rand, near_point, 30) {
                Some(sample) => {
                    let next_point_index = self.points.len();

                    self.points.push(sample);
                    self.queued.push(next_point_index);

                    self.grid.set(self.cell(sample), Some(next_point_index));
                }
                None => {
                    self.queued.remove(near_index);
                }
            }
        }
    }

    fn generate_sample(&self, rand: &mut SmallRng, near: Vec2, attempts: u32) -> Option<Vec2> {
        // Starting from a random theta angle, circle around the input point looking for an adequate nearby point.
        // https://observablehq.com/@techsparx/an-improvement-on-bridsons-algorithm-for-poisson-disc-samp/2

        let seed: f32 = rand.gen_range(0.0..PI2);

        for i in 0..attempts {
            let t = seed + map_range(i, 0, attempts, 0.0, PI2);
            let r = self.radius + 0.0001;

            let delta = vec2(t.cos(), t.sin()) * r;
            let point = near + delta;

            if !self.extent.contains(point) {
                continue;
            }

            if !self.near_point_in_grid(point) {
                return Some(point);
            }
        }

        None
    }

    /// Returns true if point [p] is near an existing point in the grid.
    fn near_point_in_grid(&self, p: Vec2) -> bool {
        assert!(self.extent.contains(p));

        let (cx, cy) = self.cell(p);

        let span = 2; // number of neighbor cells to check

        let x_min = (cx as i32 - span).max(0) as usize;
        let y_min = (cy as i32 - span).max(0) as usize;
        let x_max = (cx as i32 + span + 1).min(self.grid.cols as i32) as usize;
        let y_max = (cy as i32 + span + 1).min(self.grid.rows as i32) as usize;

        for y in y_min..y_max {
            for x in x_min..x_max {
                if self.near_point_in_cell(p, (x, y)) {
                    return true; // p is within min radius of another point
                }
            }
        }

        false
    }

    /// Returns true if point [p] is near an existing point in the specified [cell].
    fn near_point_in_cell(&self, p: Vec2, cell: (usize, usize)) -> bool {
        match self.grid.get(cell) {
            Some(i) => self.points[i].distance_squared(p) < (self.radius * self.radius),
            None => false,
        }
    }

    fn cell(&self, p: Vec2) -> (usize, usize) {
        let cx = (p.x - self.extent.x.start) / self.cell_size;
        let cy = (p.y - self.extent.y.start) / self.cell_size;
        (cx as usize, cy as usize)
    }
}
