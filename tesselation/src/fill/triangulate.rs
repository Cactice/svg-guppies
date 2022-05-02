use glam::{DVec2, Vec3};

use super::convex_breakdown::convex_breakdown;
use super::Vertex;

pub fn triangulate(polygon: &mut Vec<DVec2>, color: &Vec3) -> Vec<Vertex> {
    let mut convexes = convex_breakdown(polygon);
    convexes
        .iter_mut()
        .flat_map(|convex| {
            if convex.is_empty() {
                return vec![];
            }
            let first = convex[0];
            convex
                .iter()
                .enumerate()
                .skip(1)
                .flat_map(|(i, this)| {
                    let triangle = match convex.get(i + 1) {
                        Some(next) => vec![
                            Vertex::from((&first, color)),
                            Vertex::from((this, color)),
                            Vertex::from((next, color)),
                        ],
                        None => vec![],
                    };
                    triangle
                })
                .collect::<Vec<Vertex>>()
        })
        .collect::<Vec<Vertex>>()
}
