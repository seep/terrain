pub mod generate_erosion;
pub use generate_erosion::generate_erosion;

pub mod generate_flow;
pub use generate_flow::generate_flow;
pub use generate_flow::traverse_flow_graph;
pub use generate_flow::Flow;

pub mod generate_flux;
pub use generate_flux::generate_flux;

pub fn erode(elevation: &mut [f32], erosion: &[f32], scalar: f32) {
    for (i, e) in elevation.iter_mut().enumerate() {
        *e -= erosion[i] * scalar;
    }
}
