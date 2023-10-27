use itertools::Itertools;
use nannou::glam::Vec2;

use ordered_float::OrderedFloat;

use crate::terrain::{Terrain, VertexType};
use crate::util::{map_clamp, normalize, PriorityQueue};

pub struct Regions {
    /// The normalized habitability of each terrain vertex.
    pub habitability: Vec<f32>,
    /// The vertex index of each city.
    pub cities: Vec<usize>,
    /// The city index of each vertex.
    pub regions: Vec<usize>,
}

impl Regions {
    pub fn new(terrain: &Terrain) -> Self {
        let habitability = generate_habitability(terrain);

        let mut scores = habitability.clone();
        let mut cities = vec![];

        for _ in 0..terrain.config.num_cities {
            let city_index = scores.iter().cloned().map(OrderedFloat).position_max();
            let city_index = city_index.unwrap_or(0);

            let city_point = terrain.graph.vertices[city_index];

            // modify the score array based on the new city position

            for (i, score) in scores.iter_mut().enumerate() {
                let dist = terrain.graph.vertices[i].distance(city_point);
                *score *= map_clamp(dist, 0.0, 100.0, 0.0, 1.0);
            }

            cities.push(city_index);
        }

        let regions = generate_regions(terrain, &cities);

        Self {
            habitability,
            cities,
            regions,
        }
    }
}

fn generate_habitability(terrain: &Terrain) -> Vec<f32> {
    let mut score = vec![0.0; terrain.graph.vertices.len()];

    for (i, s) in score.iter_mut().enumerate() {
        if terrain.graph.vertex_type[i] == VertexType::Boundary {
            continue; // leave boundary vertices at 0 city score
        }

        if terrain.data.elevation[i] < 0.0 {
            continue; // leave below-sea-level vertices at 0 city score
        }

        let mut score = map_clamp(terrain.data.flux[i], 0.0, 0.05, 0.0, 1.0);

        // Scale the score towards zero near the edge (and outside) of the terrain extent.

        let vertex = terrain.graph.vertices[i];
        let extent = terrain.extent;

        let dist_x_edge = f32::min(vertex.x - extent.x.start, extent.x.end - vertex.x);
        let dist_y_edge = f32::min(vertex.y - extent.x.start, extent.y.end - vertex.y);

        score *= map_clamp(dist_x_edge, 0.0, 100.0, 0.0, 1.0);
        score *= map_clamp(dist_y_edge, 0.0, 100.0, 0.0, 1.0);

        *s = score;
    }

    normalize(&mut score);

    score
}

#[derive(Eq, PartialEq)]
struct RegionQueueValue {
    city: usize,
    vert: usize,
}

fn generate_regions(terrain: &Terrain, cities: &[usize]) -> Vec<usize> {
    let mut nearest_city = vec![None; terrain.graph.vertices.len()];

    let mut queue = PriorityQueue::new();

    // For each city, initialize the nearest_city index with itself (the nearest city of each city
    // vertex must be itself) and enqueue each of its neighboring vertices. Then process each
    // vertex in a priority queue (starting with the city neighbors) to grow each city region by
    // consuming the closest vertices first.

    for city in cities.iter().cloned() {
        nearest_city[city] = Some(city);
        for vert in terrain.graph.connected_vertices(city) {
            queue.push(
                RegionQueueValue { city, vert },
                -calculate_travel_cost(&terrain, city, vert),
            );
        }
    }

    while let Some(RegionQueueValue { city, vert }) = queue.pop() {
        if nearest_city[vert].is_some() {
            continue;
        }

        nearest_city[vert] = Some(city);

        for vert in terrain.graph.connected_vertices(vert) {
            queue.push(
                RegionQueueValue { city, vert },
                -calculate_travel_cost(&terrain, city, vert),
            );
        }
    }

    let mut region = vec![0; nearest_city.len()];

    for (i, option_city) in nearest_city.iter().enumerate() {
        region[i] = option_city.unwrap();
    }

    region
}

fn calculate_travel_cost(terrain: &Terrain, a: usize, b: usize) -> f32 {
    let pos_a = terrain.graph.vertices[a];
    let pos_b = terrain.graph.vertices[b];
    let delta_pos = Vec2::distance(pos_a, pos_b);

    let elev_a = terrain.data.elevation[a];
    let elev_b = terrain.data.elevation[b];

    // small cost for traversing water
    if elev_a < 0.0 {
        return delta_pos * 100.0;
    }

    // large cost for transitioning from land to water
    if (elev_a >= 0.0) != (elev_b >= 0.0) {
        return delta_pos * 1000.0;
    }

    let delta_elev = elev_b - elev_a;

    // uphill is less expensive than downhill (regions end on ridges)
    let delta_elev = if delta_elev > 0.0 {
        delta_elev / 10.0
    } else {
        delta_elev
    };

    let cost_elev = 0.25 * (delta_elev / delta_pos).powf(2.0);
    let cost_river = 100.0 * terrain.data.flux[a].sqrt();

    delta_pos * (1.0 + cost_elev + cost_river)
}
