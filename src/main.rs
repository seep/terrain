use std::time::Instant;

use nannou::glam::*;
use nannou::prelude::*;

mod rand;
mod regions;
mod terrain;
mod util;

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
    DebugCities,
    DebugRegions,
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
        DrawingMode::DebugRivers => DrawingMode::DebugCities,
        DrawingMode::DebugCities => DrawingMode::DebugRegions,
        DrawingMode::DebugRegions => DrawingMode::Render,
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
        DrawingMode::DebugCities => {
            debug_habitability(&draw, &model.terrain, &model.regions);
            render_cities(&draw, &model.terrain, &model.regions);
        }
        DrawingMode::DebugRegions => {
            render_terrain(&draw, &model.terrain);
            debug_regions(&draw, &model.terrain, &model.regions);
            render_cities(&draw, &model.terrain, &model.regions);
        }
        DrawingMode::Render => {
            render_terrain(&draw, &model.terrain);
            render_cities(&draw, &model.terrain, &model.regions);
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
        let p = poly.points.iter().cloned();
        let t = map_clamp(terrain.mesh.elevation[i], -500.0, 500.0, 0.0, 1.0);
        let c = colorous::COOL.eval_continuous(t as f64).into_rgb();

        draw.polygon().points(p).color(c);
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
        let p = terrain.graph.vertices[i];
        let r = map_clamp(*e, 0.0, 2.0, 0.0, 10.0);
        let t = map_clamp(*e, 0.0, 2.0, 0.0, 1.0);
        let c = colorous::MAGMA.eval_continuous(t as f64).into_rgb();

        draw.ellipse().xy(p).radius(r).color(c);
    }
}

#[allow(dead_code)]
fn debug_mesh_surface(draw: &Draw, terrain: &Terrain) {
    for (i, poly) in terrain.mesh.polygons.iter().flatten().enumerate() {
        let p = poly.points.iter().cloned();
        let c = match terrain.mesh.surface[i] {
            TerrainSurface::Water => rgb8(0, 0, 0),
            TerrainSurface::Land => rgb8(255, 255, 255),
        };

        draw.polygon().points(p).color(c);
    }
}

#[allow(dead_code)]
fn debug_rivers(draw: &Draw, terrain: &Terrain) {
    for (i, river) in terrain.mesh.rivers.iter().enumerate() {
        let p = river.points.iter().cloned();
        let c = colorous::SINEBOW.eval_rational(i % 8, 8).into_rgb();
        draw.polyline().join_round().weight(4.0).points(p).color(c);
    }
}

#[allow(dead_code)]
fn debug_habitability(draw: &Draw, terrain: &Terrain, regions: &Regions) {
    draw.background().color(BLACK);

    for (i, h) in regions.habitability.iter().cloned().enumerate() {
        let p = terrain.graph.vertices[i];
        let c = colorous::MAGMA.eval_continuous(h as f64).into_rgb();
        draw.ellipse().radius(2.0).xy(p).color(c);
    }
}

fn debug_regions(draw: &Draw, terrain: &Terrain, regions: &Regions) {
    for (i, region) in regions.regions.iter().cloned().enumerate() {
        let p = terrain.graph.vertices[i];
        let c = colorous::SINEBOW.eval_rational(region % 8, 8).into_rgb();
        draw.ellipse().radius(2.0).xy(p).color(c);
    }
}

fn render_coastline(draw: &Draw, terrain: &Terrain) {
    for (a, b) in terrain.mesh.contour.segments.iter().cloned() {
        draw.line()
            .caps_round()
            .weight(3.0)
            .points(a, b)
            .color(BLACK);
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

fn render_rivers(draw: &Draw, terrain: &Terrain) {
    for river in terrain.mesh.rivers.iter() {
        let points: Vec<Vec2> = smooth_path(&river.points).collect();
        let weight = map_clamp(river.flux, 0.005, 0.025, 3.0, 5.0);

        draw.polyline()
            .join_round()
            .weight(weight)
            .points(points)
            .color(BLACK);
    }
}

fn render_terrain(draw: &Draw, terrain: &Terrain) {
    render_coastline(draw, terrain);
    render_slopes(draw, terrain);
    render_rivers(draw, terrain);
}

fn render_cities(draw: &Draw, terrain: &Terrain, regions: &Regions) {
    for v in regions.cities.iter() {
        let p = terrain.graph.vertices[*v];

        draw.ellipse()
            .radius(4.0)
            .xy(p)
            .color(WHITE)
            .stroke_weight(2.0)
            .stroke_color(BLACK);
    }
}
