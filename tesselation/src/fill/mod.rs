mod convex_breakdown;
mod debug;
mod triangulate;
use crate::fill::triangulate::triangulate;
use crate::{Index, Vertex};
use glam::{DVec2, Vec3};
use usvg::{self, Color, PathData, PathSegment};

pub fn iterate_fill(path: &PathData, color: &Color) -> (Vec<Vertex>, Vec<Index>) {
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
        PathSegment::ClosePath => vertices.extend(triangulate(
            &mut current_polygon,
            &Vec3::new(
                color.red as f32 / u8::MAX as f32,
                color.green as f32 / u8::MAX as f32,
                color.blue as f32 / u8::MAX as f32,
            ),
        )),
    });
    (vertices, indices)
}
