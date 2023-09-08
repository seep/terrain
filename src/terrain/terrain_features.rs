use nannou::geom::*;
use nannou::rand::Rng;

use crate::rand::*;
use crate::terrain::TerrainContext;

#[derive(Debug, Clone)]
pub struct TerrainFeatures {
    pub slopes: Vec<Slope>,
    pub cones: Vec<Cone>,
    pub smooth: bool,
    pub relax: bool,
    pub erode: bool,
}

#[derive(Debug, Clone)]
pub struct Slope {
    pub origin: Vec2,
    pub direction: Vec2,
    pub length: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Cone {
    pub center: Vec2,
    pub radius: f32,
    pub height: f32,
    pub steepness: f32,
}

impl TerrainFeatures {
    /// Generate random terrain features.
    pub fn generate(context: &mut TerrainContext) -> Self {
        let expanded_extent = Rect::from_wh(context.extent.wh() * 1.2);
        let smaller_extent = Rect::from_wh(context.extent.wh() * 0.5);

        let mut slopes = vec![];
        let mut cones = vec![];

        let rand = &mut context.rand;

        // add lots of average cones

        for _ in 0..rand.gen_range(100..250) {
            let steepness = if rand.gen_bool(0.2) {
                rand.gen_range(2.0..6.0)
            } else {
                rand.gen_range(1.0..1.5)
            };

            cones.push(Cone {
                center: random_point_in_rect(rand, expanded_extent),
                radius: rand.gen_range(50.0..400.0),
                height: rand.gen_range(25.0..75.0),
                steepness,
            });
        }

        // maybe add a huge cone

        if rand.gen_bool(0.5) {
            cones.push(Cone {
                center: random_point_in_rect(rand, expanded_extent),
                radius: rand.gen_range(300.0..600.0),
                height: rand.gen_range(50.0..150.0),
                steepness: rand.gen_range(0.9..1.1),
            });
        }

        // maybe add a huge slope

        if rand.gen_bool(0.1) {
            let origin = random_point_in_rect(rand, smaller_extent);
            let direction = random_dir(rand);

            let length = rand.gen_range(100.0..300.0);
            let height = rand.gen_range(100.0..300.0);

            slopes.push(Slope {
                origin,
                direction,
                length,
                height,
            });
        }

        let smooth = rand.gen_bool(0.5);
        let relax = rand.gen_bool(0.5);
        let erode = true;

        Self {
            slopes,
            cones,
            smooth,
            relax,
            erode,
        }
    }
}
