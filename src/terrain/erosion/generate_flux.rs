use crate::terrain::erosion::{traverse_flow_graph, Flow};
use crate::terrain::TerrainGraph;

/// Generate the flux data for each vertex.
pub fn generate_flux(graph: &TerrainGraph, flow: &[Flow]) -> Vec<f32> {
    let rainfall = 1.0 / graph.vertices.len() as f32;

    let mut flux = vec![rainfall; graph.vertices.len()];

    for v in graph.interior.iter().cloned() {
        for n in traverse_flow_graph(flow, v) {
            flux[n] += rainfall;
        }
    }

    flux
}
