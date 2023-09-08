use crate::terrain::TerrainGraph;
use crate::util::PriorityQueue;

/// Generate the flow graph of the terrain vertices.
pub fn generate_flow(graph: &TerrainGraph, elevation: &[f32]) -> Vec<Flow> {
    // Implements algorithm 4 from Barnes, Lehman, Mulla [0]. Compared to the original terrain
    // generator implementation (which used Planchon-Darboux to fill depressions) this algorithm
    // does not need to modify the original elevation map. Flow nodes are generated for local
    // depressions that trace back to the local maxima. The priority queue uses the negative
    // elevation so that the lowest points are processed first.
    //
    // [0] https://arxiv.org/abs/1511.04463

    let mut flow = vec![None; graph.vertices.len()];

    let mut open = PriorityQueue::new();
    let mut seen = vec![false; flow.len()];

    for v in graph.boundary.iter().cloned() {
        open.push(v, -elevation[v]);
        seen[v] = true;
    }

    while let Some(next) = open.pop() {
        for neighbor in graph.connected_vertices(next) {
            if seen[neighbor] {
                continue;
            }

            flow[neighbor] = Some(next);
            seen[neighbor] = true;

            open.push(neighbor, -elevation[neighbor]);
        }
    }

    flow
}

pub type Flow = Option<usize>;

/// Iterate through the flow graph from an interior node to a boundary node.
pub fn traverse_flow_graph(flow: &[Flow], start: usize) -> FlowGraphIterator {
    FlowGraphIterator {
        flow,
        curr: Some(start),
    }
}

pub struct FlowGraphIterator<'a> {
    flow: &'a [Flow],
    curr: Option<usize>,
}

impl Iterator for FlowGraphIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr;

        match curr {
            Some(i) => self.curr = self.flow[i],
            None => self.curr = None,
        }

        curr
    }
}
