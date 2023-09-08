use nannou::geom::*;
use nannou::math::map_range;
use nannou::rand::rngs::SmallRng;
use nannou::rand::SeedableRng;

pub mod erosion;
pub mod terrain_data;
pub mod terrain_features;
pub mod terrain_graph;
pub mod terrain_mesh;

pub use terrain_data::TerrainData;
pub use terrain_features::TerrainFeatures;
pub use terrain_graph::TerrainGraph;
pub use terrain_graph::VertexType;
pub use terrain_mesh::TerrainMesh;

use crate::util::expand_rect;

#[derive(Debug, Clone, Copy)]
pub struct TerrainConfig {
    pub size: Vec2,
    pub seed: u64,
    pub radius: f32,
    pub num_cities: u32,
    pub num_regions: u32,
}

/// General-purpose state used for terrain generation that is derived from the config.
#[derive(Debug, Clone)]
pub struct TerrainContext {
    /// The terrain extents in world coordinates.
    pub extent: Rect,
    /// The explicitly seeded RNG used for terrain generation.
    pub rand: SmallRng,
}

#[derive(Debug, Clone)]
pub struct Terrain {
    pub config: TerrainConfig,
    /// Extent of generated terrain points.
    pub extent: Rect,
    /// The graph structures for navigating the terrain in various ways.
    pub graph: TerrainGraph,

    pub data: TerrainData,

    pub features: TerrainFeatures,

    /// The points and data used to render the final terrain.
    pub mesh: TerrainMesh,
}

pub fn generate_terrain(config: TerrainConfig) -> Terrain {
    let mut rand = SmallRng::seed_from_u64(config.seed);

    let extent = Rect::from_wh(config.size);
    let points = generate_points(&mut rand, extent, config.radius);

    let mut context = TerrainContext { extent, rand };

    let features = TerrainFeatures::generate(&mut context);

    let graph = TerrainGraph::new(&points);

    let data = TerrainData::new(&graph, &features);

    let mesh = TerrainMesh::new(&graph, &data);

    Terrain {
        config,
        extent,
        graph,
        data,
        mesh,
        features,
    }
}

/// Fill the extent with randomly sampled points, roughly separated by [radius] distance.
fn generate_points(rand: &mut SmallRng, extent: Rect, radius: f32) -> Vec<Vec2> {
    let mut points = crate::util::poisson(rand, extent, radius);

    // Generate boundary points to improve Voronoi cell generation at the edges using techniques
    // in [0]. It would be nice to skip the boundary points by clipping the boundary cells as
    // described in [1] if I can ever figure out the math.

    // [0] https://www.redblobgames.com/x/2314-poisson-with-boundary/
    // [1] https://www.microsoft.com/en-us/research/wp-content/uploads/2016/12/Efficient-Computation-of-Clipped-Voronoi-Diagram-and-Applications.pdf

    points.append(&mut generate_boundary_points(extent, radius));

    points
}

fn generate_boundary_points(extent: Rect, distance: f32) -> Vec<Vec2> {
    let inner_extent = expand_rect(extent, distance * 1.0);
    let outer_extent = expand_rect(extent, distance * 2.0);

    let mut points = vec![];

    // Add inner extent corners.

    for c in inner_extent.corners().iter() {
        points.push(Vec2::from_slice(c));
    }

    // Add outer extent corners.

    for c in outer_extent.corners().iter() {
        points.push(Vec2::from_slice(c));
    }

    // Add inner extent points.

    let min_x = inner_extent.x.start;
    let max_x = inner_extent.x.end;

    let min_y = inner_extent.y.start;
    let max_y = inner_extent.y.end;

    let nx = (inner_extent.w() / distance) as i32 - 1;
    let ny = (inner_extent.h() / distance) as i32 - 1;

    for i in 1..nx {
        let x = map_range(i, 0, nx, min_x, max_x);
        points.push(Vec2::new(x, min_y));
        points.push(Vec2::new(x, max_y));
    }

    for i in 1..ny {
        let y = map_range(i, 0, nx, min_y, max_y);
        points.push(Vec2::new(min_x, y));
        points.push(Vec2::new(max_x, y));
    }

    // Add outer extent points. Funky logic because we dont want to simply interpolate the outer
    // extents; we want n + 1 points generated at even offset from the inner boundary, so that the
    // triangles between the inner boundary and outer boundary are symmetric along an axis.

    let nx = nx + 1;
    let ny = ny + 1;

    for i in 1..nx {
        let x = map_range(i, 0, nx, min_x - distance * 0.5, max_x + distance * 0.5);
        points.push(Vec2::new(x, min_y - distance));
        points.push(Vec2::new(x, max_y + distance));
    }

    for i in 1..ny {
        let y = map_range(i, 0, ny, min_y - distance * 0.5, max_y + distance * 0.5);
        points.push(Vec2::new(min_x - distance, y));
        points.push(Vec2::new(max_x + distance, y));
    }

    points
}
