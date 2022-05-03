use glam::{DVec2, Vec4};

use super::convex_breakdown::convex_breakdown;
use super::debug::rand_vec4;
use super::Vertex;

pub fn triangulate(polygon: &mut Vec<DVec2>, color: &Vec4) -> Vec<Vertex> {
    let (mut fill, mut mask) = convex_breakdown(polygon);
    polygon.clear();
    let flattener = |color: Vec4| {
        move |convex: &mut Vec<DVec2>| {
            if convex.is_empty() {
                return vec![];
            }
            let first = &convex[0];
            convex
                .iter()
                .enumerate()
                .skip(1)
                .flat_map(|(i, this)| {
                    let triangle = match convex.get(i + 1) {
                        Some(next) => vec![
                            Vertex::from((first, &color)),
                            Vertex::from((this, &color)),
                            Vertex::from((next, &color)),
                        ],
                        None => vec![],
                    };
                    triangle
                })
                .collect::<Vec<Vertex>>()
        }
    };
    let mut fill_mapped = fill
        .iter_mut()
        .flat_map(flattener(*color))
        .collect::<Vec<Vertex>>();
    fill_mapped.extend(
        mask.iter_mut()
            .flat_map(flattener([1., 1., 1., 1.].into()))
            .collect::<Vec<Vertex>>(),
    );
    fill_mapped
}
