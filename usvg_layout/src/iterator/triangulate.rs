use glam::{vec2, DVec2, DVec3, Vec3};

use super::convex_breakdown::convex_breakdown;
use super::{Index, Vertex};

pub fn triangulate(polygon: &mut Vec<DVec2>, color: &Vec3) -> Vec<Vertex> {
    let mut convexes = convex_breakdown(polygon);
    convexes
        .iter_mut()
        .flat_map(|convex| {
            let rest = convex.split_off(1);
            let first = convex;
            rest.chunks_exact(2)
                .flat_map(|vertices| {
                    let triangle = vec![
                        Vertex::from((&first[0], color)),
                        Vertex::from((&vertices[0], color)),
                        Vertex::from((&vertices[1], color)),
                    ];
                    triangle
                })
                .collect::<Vec<Vertex>>()
        })
        .collect::<Vec<Vertex>>()
}
