use glam::{vec2, DVec2};

use super::convex_breakdown::convex_breakdown;
use super::{Index, Vertex};

pub fn triangulate(polygon: Vec<DVec2>) {
    let convexes = convex_breakdown(polygon);
    for convex in convexes {
        convex.chunks_exact(2).map();
        let triangle = Vertex::from_vec2(vec2, viewBox);
    }
}
