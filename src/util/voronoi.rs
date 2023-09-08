use delaunator::next_halfedge;
use delaunator::Triangulation;

use itertools::Itertools;

use nannou::geom::Vec2;

#[derive(Debug, Clone)]
pub struct Voronoi {
    /// The Voronoi cells. Each input point has a corresponding cell.
    pub cells: Vec<VoronoiCell>,
    /// The Voronoi cell vertices. Each vertex is the circumcenter of three input points.
    pub vertices: Vec<Vec2>,
    /// The Delaunay triangulation of the input points.
    pub triangulation: Triangulation,
}

#[derive(Debug, Clone)]
pub struct VoronoiCell {
    /// The vertices that form the cell. If this is a hull cell, these do not form a closed polygon.
    pub vertices: Vec<usize>,
    /// True if the cell surrounds a point on the convex hull.
    pub hull: bool,
}

impl Voronoi {
    pub fn new(points: &[Vec2]) -> Self {
        let triangulation = generate_triangulation(points);

        let mut vertices = Vec::with_capacity(points.len());

        for (a, b, c) in triangulation.triangles.iter().tuples() {
            vertices.push(centroid(points[*a], points[*b], points[*c]));
        }

        let incoming = build_incoming_edge_index(&triangulation);

        let mut cells = vec![];

        for halfedge in incoming.iter() {
            let mut cell_vertices = vec![];

            for e in edges_around_point(&triangulation, *halfedge) {
                cell_vertices.push(triangle_of_edge(e));
            }

            cells.push(VoronoiCell {
                vertices: cell_vertices,
                hull: false,
            });
        }

        for i in triangulation.hull.iter() {
            cells[*i].hull = true;
        }

        Self {
            cells,
            vertices,
            triangulation,
        }
    }
}

fn generate_triangulation(points: &[Vec2]) -> Triangulation {
    let mut input = vec![delaunator::Point::default(); points.len()];

    for (i, p) in points.iter().enumerate() {
        input[i].x = p.x as f64;
        input[i].y = p.y as f64;
    }

    delaunator::triangulate(&input)
}

pub fn edge_tuple_of_triangle(t: usize) -> (usize, usize, usize) {
    (t * 3, t * 3 + 1, t * 3 + 2)
}

pub fn triangle_of_edge(e: usize) -> usize {
    e / 3
}

/// Traverse the incoming edges around a point, starting with the [incoming_edge].
pub fn edges_around_point(triangulation: &Triangulation, incoming_edge: usize) -> EdgesAroundPoint {
    EdgesAroundPoint {
        triangulation,
        curr: incoming_edge,
        last: incoming_edge,
    }
}

/// Find the circumcenter of a triangle.
#[allow(dead_code)]
fn circumcenter(a: Vec2, b: Vec2, c: Vec2) -> Vec2 {
    let ad = a.length_squared();
    let bd = b.length_squared();
    let cd = c.length_squared();

    let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
    let x = 1.0 / d * (ad * (b.y - c.y) + bd * (c.y - a.y) + cd * (a.y - b.y));
    let y = 1.0 / d * (ad * (c.x - b.x) + bd * (a.x - c.x) + cd * (b.x - a.x));

    Vec2::new(x, y)
}

/// Find the centroid of a triangle.
#[allow(dead_code)]
fn centroid(a: Vec2, b: Vec2, c: Vec2) -> Vec2 {
    (a + b + c) / 3.0
}

/// Build an index from point index to some incoming edge index for edge traversal.
fn build_incoming_edge_index(triangulation: &Triangulation) -> Vec<usize> {
    let mut result = vec![delaunator::EMPTY; triangulation.triangles.len()];

    for e in 0..triangulation.triangles.len() {
        // Considering the half-edge A<-B, we can find the index of A by taking the next half-edge
        // in the loop (which is A->C) and looking at its point index in the triangles table. We
        // take the first incoming edge we find, but replace it if the incoming edge has no
        // corresponding outgoing edge; that means the the incoming edge is "leftmost" and our
        // edge traversal will visit all of the incoming edges for the point.

        let point_index = triangulation.triangles[next_halfedge(e)];
        let is_leftmost = triangulation.halfedges[e] == delaunator::EMPTY;

        if result[point_index] == delaunator::EMPTY || is_leftmost {
            result[point_index] = e;
        }
    }

    result
}

/// State struct for the edges_around_point iterator.
pub struct EdgesAroundPoint<'a> {
    triangulation: &'a Triangulation,
    curr: usize,
    last: usize,
}

impl Iterator for EdgesAroundPoint<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == delaunator::EMPTY {
            return None;
        }

        let curr = self.curr;
        let next = self.triangulation.halfedges[next_halfedge(curr)];

        if next != self.last {
            self.curr = next;
        } else {
            self.curr = delaunator::EMPTY;
        }

        Some(curr)
    }
}
