use glam::{DVec2, Vec4};

use super::convex_breakdown::convex_breakdown;
use super::debug::rand_vec4;
use super::Vertex;

pub fn triangulate(polygon: &mut Vec<DVec2>, _color: &Vec4) -> Vec<Vertex> {
    let mut convexes = convex_breakdown(polygon);
    polygon.clear();
    convexes
        .iter_mut()
        .flat_map(|convex| {
            if convex.is_empty() {
                return vec![];
            }
            let first = convex[0];
            let color_rand = &rand_vec4();
            convex
                .iter()
                .enumerate()
                .skip(1)
                .flat_map(|(i, this)| {
                    let triangle = match convex.get(i + 1) {
                        Some(next) => vec![
                            Vertex::from((&first, color_rand)),
                            Vertex::from((this, color_rand)),
                            Vertex::from((next, color_rand)),
                        ],
                        None => vec![],
                    };
                    triangle
                })
                .collect::<Vec<Vertex>>()
        })
        .collect::<Vec<Vertex>>()
}
