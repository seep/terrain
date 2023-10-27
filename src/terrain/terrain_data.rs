use nannou::geom::*;

use crate::terrain::erosion::*;
use crate::terrain::terrain_features::*;
use crate::terrain::TerrainGraph;
use crate::util::*;

#[derive(Debug, Clone)]
pub struct TerrainData {
    /// The elevation of each terrain vertex.
    pub elevation: Vec<f32>,
    /// The surface normal of each terrain vertex.
    pub normal: Vec<Vec3>,
    /// The flow of water in each terrain vertex, expressed as the index of a downhill vertex.
    pub flow: Vec<Flow>,
    /// The flux of water in each terrain vertex.
    pub flux: Vec<f32>,
    /// The erosion scalar at each terrain vertex.
    pub erosion: Vec<f32>,
}

impl TerrainData {
    pub fn new(graph: &TerrainGraph, features: &TerrainFeatures) -> Self {
        let mut elevation = vec![0f32; graph.vertices.len()];

        for feature in features.cones.iter() {
            add_elevation_cone(&mut elevation, &graph.vertices, feature);
        }

        for feature in features.slopes.iter() {
            add_elevation_slope(&mut elevation, &graph.vertices, feature);
        }

        if features.smooth {
            smooth(&mut elevation); // TODO sqrt is way too aggressive working in world coords
        }

        if features.relax {
            relax(graph, &mut elevation);
        }

        // Original implementation normalizes the elevation data here. I skip this step because it
        // introduces some oddities with mixing normalized and non-normalized data. Namely the
        // slope calculations combine the world space XY coordinates of the vertices with the
        // normalized elevation, which skews all of the slope vectors towards being very close to
        // zero. This cascades into making lots of constants for slope shading very small and
        // generally unrelated to the input data.

        // Instead we can preserve the original elevation and work in world coordinates during the
        // the slope and erosion computations. The political features (cities, towns, regions)
        // still benefit from normalized elevation data, so they calculate it there.

        let mut flow = generate_flow(graph, &elevation);
        let mut flux = generate_flux(graph, &flow);
        let mut normal = generate_normal(graph, &elevation);
        let mut erosion = generate_erosion(graph, &flux, &normal);

        for _ in 0..5 {
            erode(&mut elevation, &erosion, 500.0);

            // recalculate flow/flux/slope/erosion on each iteration
            flow = generate_flow(graph, &elevation);
            flux = generate_flux(graph, &flow);
            normal = generate_normal(graph, &elevation);
            erosion = generate_erosion(graph, &flux, &normal);
        }

        set_median_sealevel(&mut elevation);

        // TODO smooth coastline

        Self {
            elevation,
            normal,
            flow,
            flux,
            erosion,
        }
    }

    // /// Find the mean elevation of a list of vertices.
    // pub fn mean_elevation(&self, vertices: &[usize]) -> f32 {
    //     let mut sum = 0.0;
    //
    //     for v in vertices.iter() {
    //         sum += self.elevation[*v];
    //     }
    //
    //     sum / vertices.len() as f32
    // }
    //
    // /// Find the mean slope of a list of vertices.
    // pub fn mean_normal(&self, vertices: &[usize]) -> Vec3 {
    //     let mut sum = Vec3::ZERO;
    //
    //     for v in vertices.iter() {
    //         sum += self.normal[*v];
    //     }
    //
    //     sum.normalize()
    // }
}

fn add_elevation_cone(elevation: &mut [f32], points: &[Vec2], feature: &Cone) {
    // Deviation from the original work here. Instead of distinguishing between hills and cones
    // as two feature types, cones are generalized with a steepness parameter that introduces an
    // exponential falloff. A steepness of 1 creates a linear falloff, increasing steepness
    // produces a falloff with exponential in-out easing.

    for (i, p) in points.iter().cloned().enumerate() {
        let d = p - feature.center;
        let t = saturate(1.0 - d.length() / feature.radius);
        let t = ease_with_power(t, feature.steepness);

        elevation[i] += feature.height * t;
    }
}

fn add_elevation_slope(elevation: &mut [f32], points: &[Vec2], feature: &Slope) {
    // I believe mewo generated all slopes as bisecting the center the of extents, and rlguy used
    // something closer to the implementation below, which generates slopes with random origin.

    let slope = feature.direction * feature.length;
    let lensq = slope.length_squared();

    for (i, p) in points.iter().cloned().enumerate() {
        let d = p - feature.origin;
        let t = saturate(d.dot(slope) / lensq);

        elevation[i] += feature.height * t;
    }
}

/// Take the square root of each elevation.
fn smooth(elevation: &mut [f32]) {
    for e in elevation.iter_mut() {
        *e = e.sqrt();
    }
}

/// Replace each elevation with the average of its neighbors.
fn relax(graph: &TerrainGraph, elevation: &mut [f32]) {
    let mut average = elevation.to_owned();

    for (i, a) in average.iter_mut().enumerate() {
        let mut sum = 0.0;
        let mut div = 0.0;

        for n in graph.connected_vertices(i) {
            sum += elevation[n];
            div += 1.0;
        }

        if div > 0.0 {
            *a = sum / div;
        } else {
            *a = 0.0;
        }
    }

    elevation.clone_from_slice(&average);
}

/// Replace each elevation with its difference from the [sealevel] elevation.
fn set_sealevel(elevation: &mut [f32], sealevel: f32) {
    for e in elevation.iter_mut() {
        *e -= sealevel;
    }
}

fn set_median_sealevel(elevation: &mut [f32]) {
    let median = median(elevation);
    set_sealevel(elevation, median);
}

fn median(elevation: &[f32]) -> f32 {
    let mut sorted = elevation.to_owned();

    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let sorted_len = sorted.len();
    let sorted_mid = sorted_len / 2;

    if sorted_len % 2 == 0 {
        return (sorted[sorted_mid] + sorted[sorted_mid + 1]) * 0.5;
    }

    sorted[sorted_mid]
}

/// Find the surface normal of each terrain vertex.
fn generate_normal(graph: &TerrainGraph, elevation: &[f32]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; elevation.len()];

    for v in graph.interior.iter() {
        let (na, nb, nc) = graph.interior_connected_vertices(*v).unwrap();

        let pa = Vec3::from((graph.vertices[na], elevation[na]));
        let pb = Vec3::from((graph.vertices[nb], elevation[nb]));
        let pc = Vec3::from((graph.vertices[nc], elevation[nc]));

        let normal = Vec3::cross(pb - pa, pc - pa).normalize_or_zero();

        normals[*v] = normal;
    }

    normals
}

fn ease_with_power(t: f32, p: f32) -> f32 {
    // generalized exponential easing https://www.s-ings.com/scratchpad/exponential-easing/

    if t <= 0.5 {
        (t * 2.0).powf(p) * 0.5
    } else {
        1.0 - (2.0 - t * 2.0).powf(p) * 0.5
    }
}
