use nannou::glam::*;

use crate::terrain::TerrainGraph;

const EROSION_MIN: f32 = 0.00;
const EROSION_MAX: f32 = 0.02;

pub fn generate_erosion(graph: &TerrainGraph, flux: &[f32], normals: &[Vec3]) -> Vec<f32> {
    let mut erosion = vec![0f32; graph.vertices.len()];

    for (i, e) in erosion.iter_mut().enumerate() {
        let scalar = normals[i].xy().length_squared();
        let river = scalar * flux[i].sqrt();
        let creep = scalar * 0.001;

        *e = (river + creep).clamp(EROSION_MIN, EROSION_MAX);
    }

    erosion
}
