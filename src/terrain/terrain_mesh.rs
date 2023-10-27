use nannou::glam::*;
use nannou::math::*;
use nannou::rand::random;

use crate::terrain::erosion::traverse_flow_graph;
use crate::terrain::{TerrainData, TerrainGraph};
use crate::util::{indexed_mean, map_clamp};

#[derive(Debug, Clone)]
pub struct TerrainMesh {
    /// The polygon of each terrain cell. No polygons are generated for cells of hull points.
    pub polygons: Vec<Option<TerrainPolygon>>,
    /// The contour of the terrain coastline.
    pub contour: TerrainContour,
    /// Line segments to shade slopes.
    pub shading: Vec<TerrainShading>,

    pub rivers: Vec<TerrainRiver>,

    /// The elevation of each terrain polygon, as the mean of its vertices.
    pub elevation: Vec<f32>,
    /// The surface type of each terrain polygon.
    pub surface: Vec<TerrainSurface>,
}

#[derive(Debug, Clone)]
pub struct TerrainPolygon {
    /// The points composing the polygon.
    pub points: Vec<Vec2>,
}

#[derive(Debug, Clone)]
pub struct TerrainShading {
    pub points: (Vec2, Vec2),
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct TerrainContour {
    pub segments: Vec<(Vec2, Vec2)>,
    /// True if a particular vertex is on the contour.
    pub is_contour: Vec<bool>,
    /// True if a particular vertex is on or inside the contour.
    pub is_surface: Vec<bool>,
}

#[derive(Debug, Clone)]
pub struct TerrainRiver {
    /// A sequential list of points comprising the river segment.
    pub points: Vec<Vec2>,
    /// The mean flux across the river segment.
    pub flux: f32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TerrainSurface {
    Water,
    Land,
}

impl TerrainMesh {
    pub fn new(graph: &TerrainGraph, data: &TerrainData) -> Self {
        let polygons = generate_polygons(graph);

        // Compute the mean elevation of each terrain polygon.

        let mut elevation = vec![0.0; polygons.len()];

        for i in 0..polygons.len() {
            elevation[i] = indexed_mean(&data.elevation, graph.cell(i));
        }

        // Compute the mean surface normal of each terrain polygon.

        let mut normals = vec![Vec3::Y; polygons.len()];

        for (i, _) in polygons.iter().flatten().enumerate() {
            let mut normal = Vec3::ZERO;

            for v in graph.cell(i) {
                normal += data.normal[*v];
            }

            normals[i] = normal.normalize_or_zero();
        }

        // Classify each terrain polygon into a surface type.

        let mut surface = vec![TerrainSurface::Water; polygons.len()];

        for i in 0..polygons.iter().len() {
            if elevation[i] >= 0.0 {
                surface[i] = TerrainSurface::Land;
            }
        }

        let shading = generate_shading(graph, &surface, &normals);
        let contour = generate_contour(graph, &surface);

        let rivers = generate_rivers(graph, data, &contour);

        Self {
            polygons,
            elevation,
            surface,
            contour,
            shading,
            rivers,
        }
    }
}

fn generate_polygons(graph: &TerrainGraph) -> Vec<Option<TerrainPolygon>> {
    let mut polygons = vec![None; graph.points.len()];

    for (i, poly) in polygons.iter_mut().enumerate() {
        if graph.is_hull_cell(i) {
            continue;
        }

        let mut points = vec![];

        for v in graph.cell(i) {
            points.push(graph.vertices[*v])
        }

        *poly = Some(TerrainPolygon { points });
    }

    polygons
}

fn generate_contour(graph: &TerrainGraph, surface: &[TerrainSurface]) -> TerrainContour {
    let mut segments = vec![];
    let mut is_contour = vec![false; graph.vertices.len()];

    for edge in graph.edges.iter() {
        if surface[edge.points.0] != surface[edge.points.1] {
            is_contour[edge.vertices.0] = true;
            is_contour[edge.vertices.1] = true;

            let va = graph.vertices[edge.vertices.0];
            let vb = graph.vertices[edge.vertices.1];

            segments.push((va, vb));
        }
    }

    let mut is_surface = is_contour.clone();

    for i in 0..graph.points.len() {
        if surface[i] == TerrainSurface::Land {
            for vert in graph.cell(i) {
                is_surface[*vert] = true;
            }
        }
    }

    TerrainContour {
        segments,
        is_contour,
        is_surface,
    }
}

fn generate_rivers(
    graph: &TerrainGraph,
    data: &TerrainData,
    contour: &TerrainContour,
) -> Vec<TerrainRiver> {
    // Construct a list of vertex indices which will compose the rivers. These vertices are on the
    // surface (on or inside the contour) and have sufficient water flux. I sort the vertices by
    // their flux from lowest to highest; as we construct the river segments, this means each
    // segment starts at its most inland nod, which prevents discontinuities.

    let mut indices = vec![];

    for v in graph.interior.iter() {
        if contour.is_surface[*v] && data.flux[*v] >= 0.005 {
            indices.push(*v);
        }
    }

    indices.sort_by(|a, b| f32::partial_cmp(&data.flux[*a], &data.flux[*b]).unwrap());

    let mut seen = vec![false; graph.vertices.len()];
    let mut rivers = vec![];

    for v in indices {
        let mut points = vec![];
        let mut flux = 0.0;

        for n in traverse_flow_graph(&data.flow, v) {
            points.push(graph.vertices[n]);
            flux += data.flux[n];

            if contour.is_contour[n] {
                break; // terminate after we reach the contour
            }

            if seen[n] {
                break; // terminate after we reach a vertex we have seen
            }

            seen[n] = true;
        }

        flux /= points.len() as f32;

        rivers.push(TerrainRiver { points, flux });
    }

    rivers
}

const SHADING_LIGHT_THRESHOLD: f32 = 0.25;
const SLOPE_SHADING_STEEPNESS: f32 = 1.0;

fn generate_shading(
    graph: &TerrainGraph,
    surface: &[TerrainSurface],
    normals: &[Vec3],
) -> Vec<TerrainShading> {
    let mut shading = vec![];

    let light = vec3(1.0, -1.0, -1.0).normalize();

    // This section is significantly different than the original implementation...I couldnt
    // grok the code. But it arrives at a similar style. First do a standard lighting pass by
    // taking the dot product of the 3D surface normal against a 3D light vector and normalizing
    // it. This produces a shading value that, when above a threshold, can be mapped to stroke
    // weight and length in a straightforward way. Orient the strokes with the elevation
    // gradient as in the Hachure style [0].
    //
    // [0] https://en.wikipedia.org/wiki/Hachure_map

    for (i, point) in graph.points.iter().enumerate() {
        if surface[i] == TerrainSurface::Water {
            continue;
        }

        let normal = normals[i];
        let shadow = normal.dot(-light) * 0.5 + 0.5;

        if shadow < SHADING_LIGHT_THRESHOLD {
            continue;
        }

        let t = map_range(shadow, SHADING_LIGHT_THRESHOLD, 1.0, 0.0, 1.0);

        let angle = normal.x * SLOPE_SHADING_STEEPNESS;
        let angle = angle + map_range(random(), 0.0, 1.0, -0.1, 0.1);

        let stroke = vec2(angle.cos(), angle.sin());

        let length = map_clamp(t, 0.0, 1.0, 2.0, 6.0);
        let weight = map_clamp(t, 0.0, 1.0, 1.0, 3.0);

        let offset = vec2(stroke.y, -stroke.x) * map_range(t, 0.0, 1.0, 1.0, 2.0);

        let pa = *point;
        let pb = *point + stroke * length;

        shading.push(TerrainShading {
            points: (pa - offset, pb - offset),
            weight,
        });

        shading.push(TerrainShading {
            points: (pa + offset, pb + offset),
            weight,
        });
    }

    shading
}
