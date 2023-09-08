use itertools::Itertools;

use ordered_float::OrderedFloat;

use crate::terrain::{Terrain, VertexType};
use crate::util::{map_clamp, normalize, PriorityQueue};

pub struct Regions {
    /// The normalized habitability of each terrain vertex.
    pub habitability: Vec<f32>,
    /// The vertices of each city.
    pub cities: Vec<usize>,
}

impl Regions {
    pub fn new(terrain: &Terrain) -> Self {
        let habitability = generate_habitability(terrain);

        let mut scores = habitability.clone();
        let mut cities = vec![];

        for i in 0..terrain.config.num_cities {
            let city_index = scores.iter().cloned().map(OrderedFloat).position_max();
            let city_index = city_index.unwrap_or(0);

            let city_point = terrain.graph.vertices[city_index];

            for (j, score) in scores.iter_mut().enumerate() {
                let dist = terrain.graph.vertices[j].distance(city_point);
                *score *= map_clamp(dist, 0.0, 100.0, 0.0, 1.0);
            }

            cities.push(city_index);
        }

        Self {
            habitability,
            cities,
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

fn generate_regions(terrain: &Terrain, cities: &[usize]) {
    let mut nearest_city = vec![None; terrain.graph.points.len()];

    let mut queue = PriorityQueue::new();

    fn calculate_travel_cost(a: usize, b: usize) -> f32 {
        0f32
    }

    // For each city, initialize the nearest_city index with itself (the nearest city of each city
    // vertex must be itself) and enqueue each of its neighboring vertices.

    for city in cities.iter().cloned() {
        nearest_city[city] = Some(city);
        for vert in terrain.graph.connected_vertices(city) {
            queue.push(
                RegionQueueValue { city, vert },
                calculate_travel_cost(city, vert),
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
                calculate_travel_cost(city, vert),
            );
        }
    }
}
