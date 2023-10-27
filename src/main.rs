use itertools::Itertools;
use nannou::color::IntoLinSrgba;
use std::time::Instant;

use nannou::glam::*;
use nannou::prelude::*;

mod rand;
mod regions;
mod terrain;
mod util;

use crate::terrain::terrain_mesh::{TerrainPolygon, TerrainSurface};
use regions::*;
use terrain::*;
use util::*;

const SIZE_X: u32 = 1000;
const SIZE_Y: u32 = 1000;

struct Model {
    terrain: Terrain,
    regions: Regions,
    mode: DrawingMode,
}

fn main() {
    nannou::app(model).view(view).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(SIZE_X, SIZE_Y)
        .view(view)
        .mouse_released(mouse_released)
        .build()
        .unwrap();

    let config = TerrainConfig {
        size: Vec2::new(SIZE_X as f32, SIZE_Y as f32),
        seed: random(),
        radius: 10.0,
        num_cities: 5,
    };

    let terrain = generate_terrain(config);
    let regions = Regions::new(&terrain);

    Model {
        terrain,
        regions,
        mode: DrawingMode::Render,
    }
}

#[derive(Debug, Copy, Clone)]
enum DrawingMode {
    DebugMesh,
    DebugGraphVerts,
    DebugGraphEdges,
    DebugElevation,
    DebugSlope,
    DebugFlow,
    DebugErosion,
    DebugRivers,
    DebugCityScore,
    Render,
}

fn cycle_drawing_mode(mode: DrawingMode) -> DrawingMode {
    match mode {
        DrawingMode::DebugMesh => DrawingMode::DebugGraphVerts,
        DrawingMode::DebugGraphVerts => DrawingMode::DebugGraphEdges,
        DrawingMode::DebugGraphEdges => DrawingMode::DebugElevation,
        DrawingMode::DebugElevation => DrawingMode::DebugSlope,
        DrawingMode::DebugSlope => DrawingMode::DebugFlow,
        DrawingMode::DebugFlow => DrawingMode::DebugErosion,
        DrawingMode::DebugErosion => DrawingMode::DebugRivers,
        DrawingMode::DebugRivers => DrawingMode::DebugCityScore,
        DrawingMode::DebugCityScore => DrawingMode::Render,
        DrawingMode::Render => DrawingMode::DebugMesh,
    }
}

fn mouse_released(_: &App, model: &mut Model, button: MouseButton) {
    if button == MouseButton::Left {
        let now = Instant::now();

        let mut config = model.terrain.config.clone();

        config.seed = random();

        model.terrain = generate_terrain(config);
        model.regions = Regions::new(&model.terrain);

        let npoints = model.terrain.graph.points.len();
        let elapsed = now.elapsed();

        println!(
            "generated terrain with {:?} points in {:.4?}",
            npoints, elapsed,
        );
    }

    if button == MouseButton::Right {
        model.mode = cycle_drawing_mode(model.mode);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(SNOW);

    match model.mode {
        DrawingMode::DebugMesh => {
            debug_mesh_polygons(&draw, &model.terrain);
        }
        DrawingMode::DebugGraphVerts => {
            debug_graph_vertices(&draw, &model.terrain);
        }
        DrawingMode::DebugGraphEdges => {
            debug_graph_edges(&draw, &model.terrain);
        }
        DrawingMode::DebugElevation => {
            debug_elevation(&draw, &model.terrain);
        }
        DrawingMode::DebugSlope => {
            debug_elevation(&draw, &model.terrain);
            debug_normal(&draw, &model.terrain);
        }
        DrawingMode::DebugFlow => {
            debug_elevation(&draw, &model.terrain);
            debug_flow(&draw, &model.terrain);
        }
        DrawingMode::DebugErosion => {
            debug_elevation(&draw, &model.terrain);
            debug_erosion(&draw, &model.terrain);
        }
        DrawingMode::DebugRivers => {
            debug_mesh_surface(&draw, &model.terrain);
            debug_rivers(&draw, &model.terrain);
        }
        DrawingMode::DebugCityScore => {
            debug_elevation(&draw, &model.terrain);
            debug_cities(&draw, &model.terrain, &model.regions);
        }
        DrawingMode::Render => {
            render_coastline(&draw, &model.terrain);
            render_rivers(&draw, &model.terrain);
            render_slopes(&draw, &model.terrain);
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

#[allow(dead_code)]
fn debug_points(draw: &Draw, terrain: &Terrain) {
    for p in terrain.graph.points.iter() {
        draw.ellipse().radius(2.0).color(RED).xy(*p);
    }
}

#[allow(dead_code)]
fn debug_graph_vertices(draw: &Draw, terrain: &Terrain) {
    for (i, v) in terrain.graph.vertices.iter().enumerate() {
        let color = match terrain.graph.vertex_type[i] {
            VertexType::Boundary => MAGENTA,
            VertexType::Interior => GREENYELLOW,
        };

        draw.ellipse().radius(2.0).color(color).xy(*v);
    }
}

#[allow(dead_code)]
fn debug_graph_edges(draw: &Draw, terrain: &Terrain) {
    for e in terrain.graph.edges.iter() {
        let va = terrain.graph.vertices[e.vertices.0];
        let vb = terrain.graph.vertices[e.vertices.1];

        let pa = terrain.graph.points[e.points.0];
        let pb = terrain.graph.points[e.points.1];

        let edge_middle = Vec2::lerp(va, vb, 0.5);
        let edge_normal = (pb - pa).normalize();

        let cross_length = terrain.config.radius * 0.2;
        let ca = edge_middle + edge_normal * cross_length;
        let cb = edge_middle - edge_normal * cross_length;

        draw.line().points(ca, cb).color(DIMGREY);
        draw.line().points(va, vb).color(DIMGREY);
    }
}

#[allow(dead_code)]
fn debug_mesh_polygons(draw: &Draw, terrain: &Terrain) {
    for poly in terrain.mesh.polygons.iter().flatten() {
        let points = poly.points.iter().cloned();
        draw.polyline().points(points).color(DIMGREY);
    }
}

#[allow(dead_code)]
fn debug_elevation(draw: &Draw, terrain: &Terrain) {
    for (i, poly) in terrain.mesh.polygons.iter().flatten().enumerate() {
        let t = map_clamp(terrain.mesh.elevation[i], -500.0, 500.0, 0.0, 1.0);
        let c = colorous::COOL.eval_continuous(t as f64);
        draw_polygon(draw, poly, c.as_tuple());
    }
}

#[allow(dead_code)]
fn debug_normal(draw: &Draw, terrain: &Terrain) {
    for (i, p) in terrain.graph.points.iter().enumerate() {
        let mut n = Vec3::ZERO;

        for v in terrain.graph.cell(i) {
            n += terrain.data.normal[*v];
        }

        let pa = *p;
        let pb = *p + n.normalize().xy() * terrain.config.radius * 0.5;

        draw.line()
            .caps_round()
            .weight(2.0)
            .color(RED)
            .points(pa, pb);
    }
}

#[allow(dead_code)]
fn debug_flow(draw: &Draw, terrain: &Terrain) {
    for (i, v) in terrain.graph.vertices.iter().enumerate() {
        if let Some(next) = terrain.data.flow[i] {
            let pa = *v;
            let pb = terrain.graph.vertices[next];

            let w = map_clamp(terrain.data.flux[i], 0.0, 1.0, 1.0, 10.0);

            draw.line()
                .caps_round()
                .weight(w)
                .color(BLACK)
                .points(pa, pb);
        }
    }
}

#[allow(dead_code)]
fn debug_erosion(draw: &Draw, terrain: &Terrain) {
    for (i, e) in terrain.data.erosion.iter().enumerate() {
        let vertex = terrain.graph.vertices[i];
        let radius = map_clamp(*e, 0.0, 2.0, 0.0, 10.0);

        let t = map_clamp(*e, 0.0, 2.0, 0.0, 1.0);
        let c = colorous::MAGMA.eval_continuous(t as f64);
        let c = Rgb::from(c.as_tuple());

        draw.ellipse().xy(vertex).radius(radius).color(c);
    }
}

#[allow(dead_code)]
fn debug_mesh_surface(draw: &Draw, terrain: &Terrain) {
    for (i, poly) in terrain.mesh.polygons.iter().flatten().enumerate() {
        let color = match terrain.mesh.surface[i] {
            TerrainSurface::Water => (0, 0, 0),
            TerrainSurface::Land => (255, 255, 255),
        };

        draw_polygon(draw, poly, color);
    }
}

#[allow(dead_code)]
fn debug_rivers(draw: &Draw, terrain: &Terrain) {
    for (i, r) in terrain.mesh.rivers.iter().enumerate() {
        let t = map_range(i % 8, 0, 8, 0.0, 1.0);
        let c = colorous::SINEBOW.eval_continuous(t);
        let c = Rgb::from(c.as_tuple());

        draw.polyline()
            .color(c)
            .weight(4.0)
            .points(r.points.iter().cloned());
    }
}

fn debug_cities(draw: &Draw, terrain: &Terrain, regions: &Regions) {
    for (i, poly) in terrain.mesh.polygons.iter().flatten().enumerate() {
        let t = indexed_mean(&regions.habitability, terrain.graph.cell(i));
        let c = colorous::MAGMA.eval_continuous(t as f64).as_tuple();
        draw_polygon(draw, poly, c);
    }

    for v in regions.cities.iter() {
        let p = terrain.graph.vertices[*v];
        draw.ellipse().xy(p).radius(4.0).color(WHITE);
    }
}

fn render_coastline(draw: &Draw, terrain: &Terrain) {
    for (a, b) in terrain.mesh.contour.segments.iter().cloned() {
        draw.line()
            .caps_round()
            .color(BLACK)
            .weight(3.0)
            .points(a, b);
    }
}

fn render_rivers(draw: &Draw, terrain: &Terrain) {
    for river in terrain.mesh.rivers.iter() {
        let points: Vec<Vec2> = smooth_path(&river.points).collect();
        let weight = map_clamp(river.flux, 0.005, 0.025, 3.0, 5.0);

        draw.polyline()
            .join_round()
            .color(BLACK)
            .weight(weight)
            .points(points);
    }
}

fn render_slopes(draw: &Draw, terrain: &Terrain) {
    for shading in terrain.mesh.shading.iter() {
        let w = shading.weight;
        let a = shading.points.0;
        let b = shading.points.1;

        draw.line().caps_round().color(BLACK).weight(w).points(a, b);
    }
}

fn draw_polygon(draw: &Draw, poly: &TerrainPolygon, color: (u8, u8, u8)) {
    let points = poly.points.iter().cloned();
    draw.polygon().points(points).color(Rgb::from(color));
}
