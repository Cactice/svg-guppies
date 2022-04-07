mod line_to_parallel_lines;
use glam::DVec2;
use line_to_parallel_lines::line_to_parallel_lines;
use usvg::{self, PathData, PathSegment, ViewBox};

pub type Index = u32;
pub fn iterate(path: &PathData, width: f64, viewBox: &ViewBox) -> (Vec<Vertex>, Vec<Index>) {
    let mut vertices: Vec<Vertex> = vec![];
    let mut indices: Vec<Index> = vec![];
    let mut current_vec2: DVec2 = DVec2::new(0.0, 0.0);
    // println!(
    //     "viewBox x:{}, y:{}",
    //     viewBox.rect.width(),
    //     viewBox.rect.height()
    // );

    path.iter().for_each(|path| match path {
        PathSegment::MoveTo { x, y } => {
            current_vec2 = DVec2::new(*x, *y);
        }
        PathSegment::LineTo { x, y } => {
            // Below wiki is a reference of what is being done here
            // https://github.com/nical/lyon/wiki/Stroke-tessellation
            let next_vec2 = DVec2::new(*x, *y);
            let ((p0, p1), (p2, p3)) = line_to_parallel_lines((current_vec2, next_vec2), width);
            let rect = [p0, p1, p2, p1, p2, p3];
            let new_vertices = rect.iter().map(|vec2| Vertex::from_vec2(vec2, viewBox));
            vertices.extend(new_vertices);
            let len = vertices.len() as u32;
            // indices pattern to create two triangles that make a rectangle
            let new_indices = [4, 3, 2, 3, 2, 1].iter().map(|index_diff| len - index_diff);
            indices.extend(new_indices);
            current_vec2 = next_vec2;
        }
        PathSegment::CurveTo {
            x1: _,
            y1: _,
            x2: _,
            y2: _,
            x,
            y,
        } => {
            // TODO: This is not curving at all
            let next_vec2 = DVec2::new(*x, *y);
            let ((p0, p1), (p2, p3)) = line_to_parallel_lines((current_vec2, next_vec2), width);
            let rect = [p0, p1, p2, p1, p2, p3];
            let new_vertices = rect.iter().map(|vec2| Vertex::from_vec2(vec2, viewBox));
            vertices.extend(new_vertices);
            let len = vertices.len() as u32;
            // indices pattern to create two triangles that make a rectangle
            let new_indices = [4, 3, 2, 3, 2, 1].iter().map(|index_diff| len - index_diff);
            indices.extend(new_indices);
            current_vec2 = next_vec2;
        }
        PathSegment::ClosePath => {}
    });
    (vertices, indices)
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub _padding1: f32,
    pub color: [f32; 3],
    pub _padding2: f32,
}
impl Vertex {
    fn from_vec2(v: &DVec2, view_box: &ViewBox) -> Self {
        Self {
            position: [
                (v.x / view_box.rect.width()) as f32,
                (-v.y / view_box.rect.height()) as f32,
                0.0,
            ],
            ..Default::default()
        }
    }
}
