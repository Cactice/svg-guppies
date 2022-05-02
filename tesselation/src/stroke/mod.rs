mod line_to_parallel_lines;
use glam::DVec2;
use line_to_parallel_lines::line_to_parallel_lines;
use usvg::{self, PathData, PathSegment};

use crate::{Index, Vertex};

pub fn iterate_stroke(path: &PathData, width: f64) -> (Vec<Vertex>, Vec<Index>) {
    let mut vertices: Vec<Vertex> = vec![];
    let indices: Vec<Index> = vec![];
    let mut current_vec2: DVec2 = DVec2::new(0.0, 0.0);

    let mut draw_line = |x: &f64, y: &f64, current_vec2: &DVec2| {
        // Below wiki is a reference of what is being done here
        // https://github.com/nical/lyon/wiki/Stroke-tessellation
        let next_vec2 = DVec2::new(*x, *y);
        let ((p0, p1), (p2, p3)) = line_to_parallel_lines((*current_vec2, next_vec2), width);
        let rect = [p0, p1, p2, p1, p2, p3];
        let new_vertices = rect.iter().map(Vertex::from);
        vertices.extend(new_vertices);
        let _len = vertices.len() as u32;
        // indices pattern to create two triangles that make a rectangle
        // let new_indices = [4, 3, 2, 3, 2, 1].iter().map(|index_diff| len - index_diff);
        // indices.extend(new_indices);
        next_vec2
    };
    path.iter().for_each(|path| match path {
        PathSegment::MoveTo { x, y } => {
            current_vec2 = DVec2::new(*x, *y);
        }
        PathSegment::LineTo { x, y } => current_vec2 = draw_line(x, y, &current_vec2),
        PathSegment::CurveTo {
            x1: _,
            y1: _,
            x2: _,
            y2: _,
            x,
            y,
        } => {
            // TODO: This is not curving at all
            current_vec2 = draw_line(x, y, &current_vec2);
        }
        PathSegment::ClosePath => {}
    });
    (vertices, indices)
}
