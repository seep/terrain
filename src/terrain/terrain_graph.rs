use nannou::geom::*;

use crate::util::voronoi;
use crate::util::voronoi::Voronoi;

#[derive(Debug, Clone)]
pub struct TerrainGraph {
    /// The terrain control points (ie delauney points).
    pub points: Vec<Vec2>,
    /// The terrain vertices.
    pub vertices: Vec<Vec2>,
    /// The indices of the boundary vertices.
    pub boundary: Vec<usize>,
    /// The indices of the interior (non-boundary) vertices.
    pub interior: Vec<usize>,
    /// The type of each vertex.
    pub vertex_type: Vec<VertexType>,
    /// The terrain edges.
    pub edges: Vec<TerrainGraphEdge>,
    /// The Voronoi tesselation backing the terrain graph.
    voronoi: Voronoi,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VertexType {
    Interior,
    Boundary,
}

#[derive(Debug, Copy, Clone)]
pub struct TerrainGraphEdge {
    /// The indices of the vertices forming the edge.
    pub vertices: (usize, usize),
    /// The indices of the input points adjacent to the edge.
    pub points: (usize, usize),
}

impl TerrainGraph {
    pub fn new(points: &Vec<Vec2>) -> Self {
        // Generate the Voronoi tesselation for the input points.

        let voronoi = Voronoi::new(points);

        // Copy the voronoi vertices,

        let vertices = voronoi.vertices.clone();

        // Construct a lookup from vertex index to vertex type.

        let mut vertex_type = vec![VertexType::Interior; vertices.len()];

        for i in voronoi.triangulation.hull.iter() {
            for v in voronoi.cells[*i].vertices.iter() {
                vertex_type[*v] = VertexType::Boundary;
            }
        }

        // Construct two subsets with the indices of either the interior and boundary vertices.

        let mut boundary = vec![];
        let mut interior = vec![];

        for (i, _) in vertices.iter().enumerate() {
            if vertex_type[i] == VertexType::Boundary {
                boundary.push(i);
            } else {
                interior.push(i);
            }
        }

        // Construct a lookup from vertex index to connected vertex indices. I only generate
        // graph entries for interior vertices; the connections of the boundary vertices are
        // never used.

        let mut edges = Vec::with_capacity(voronoi.triangulation.triangles.len());

        let mut halfedge_seen = vec![false; voronoi.triangulation.triangles.len()];

        for i in 0..voronoi.triangulation.triangles.len() {
            let inc_halfedge = i;
            let out_halfedge = voronoi.triangulation.halfedges[i];

            if halfedge_seen[inc_halfedge] {
                continue;
            }

            if out_halfedge == delaunator::EMPTY {
                continue;
            }

            halfedge_seen[inc_halfedge] = true;
            halfedge_seen[out_halfedge] = true;

            let va = voronoi::triangle_of_edge(out_halfedge);
            let vb = voronoi::triangle_of_edge(inc_halfedge);
            let vertices = (va, vb);

            let pa = voronoi.triangulation.triangles[out_halfedge];
            let pb = voronoi.triangulation.triangles[inc_halfedge];
            let points = (pa, pb);

            edges.push(TerrainGraphEdge { vertices, points });
        }

        Self {
            points: points.clone(),
            vertices,
            boundary,
            interior,
            vertex_type,
            edges,
            voronoi,
        }
    }

    /// Get the vertices forming the Voronoi cell around input point [p].
    pub fn cell(&self, p: usize) -> &[usize] {
        self.voronoi.cells[p].vertices.as_slice()
    }

    pub fn is_hull_cell(&self, p: usize) -> bool {
        self.voronoi.cells[p].hull
    }

    /// Iterate over the vertex indices connected to vertex [v].
    pub fn connected_vertices(&self, v: usize) -> ConnectedVerticesIterator {
        ConnectedVerticesIterator {
            voronoi: &self.voronoi,
            vertex: v,
            offset: 0,
        }
    }

    /// Get a triplet tuple of connected vertex indices for an interior vertex. Returns None if the vertex is a boundary vertex.
    pub fn interior_connected_vertices(&self, v: usize) -> Option<(usize, usize, usize)> {
        // We can find the vertex neighbors by finding the three half-edges that compose the
        // corresponding triangle (each Voronoi vertex is the center of a Delaunay triangle). For
        // each half-edge, we find its opposite half-edge in the triangulation. If the opposite
        // half-edge is empty, no neighboring vertex exists.

        if self.vertex_type[v] == VertexType::Boundary {
            return None;
        }

        let (ea, eb, ec) = voronoi::edge_tuple_of_triangle(v);

        let ha = self.voronoi.triangulation.halfedges[ea];
        let hb = self.voronoi.triangulation.halfedges[eb];
        let hc = self.voronoi.triangulation.halfedges[ec];

        assert!(ha != delaunator::EMPTY && hb != delaunator::EMPTY && hc != delaunator::EMPTY);

        let ta = voronoi::triangle_of_edge(ha);
        let tb = voronoi::triangle_of_edge(hb);
        let tc = voronoi::triangle_of_edge(hc);

        Some((ta, tb, tc))
    }
}

pub struct ConnectedVerticesIterator<'a> {
    voronoi: &'a Voronoi,
    /// The vertex to iterate around.
    vertex: usize,
    /// The current half-edge index (0/1/2).
    offset: usize,
}

impl Iterator for ConnectedVerticesIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.offset < 3 {
            let incoming = self.vertex * 3 + self.offset;
            let outgoing = self.voronoi.triangulation.halfedges[incoming];

            self.offset += 1;

            if outgoing == delaunator::EMPTY {
                continue;
            }

            return Some(voronoi::triangle_of_edge(outgoing));
        }

        None
    }
}
