mod convex_breakdown;
#[allow(dead_code)]
mod debug;
mod triangulate;
use crate::fill::triangulate::triangulate;
use crate::{Index, Vertex};
use glam::{DVec2, Vec4};
use usvg::{self, PathData, PathSegment};

pub fn iterate_fill(path: &PathData, color: &Vec4) -> (Vec<Vertex>, Vec<Index>) {
    let mut vertices: Vec<Vertex> = vec![];
    let indices: Vec<Index> = vec![];
    let mut current_polygon: Vec<DVec2> = vec![];

    path.iter().for_each(|path| match path {
        PathSegment::MoveTo { x, y } => {
            current_polygon.push(DVec2::new(*x, *y));
        }
        PathSegment::LineTo { x, y } => current_polygon.push(DVec2::new(*x, *y)),
        PathSegment::CurveTo {
            x1: _,
            y1: _,
            x2: _,
            y2: _,
            x,
            y,
        } => {
            // TODO: This is not curving at all
            current_polygon.push(DVec2::new(*x, *y));
        }
        PathSegment::ClosePath => vertices.extend(triangulate(&mut current_polygon, color)),
    });
    (vertices, indices)
}
