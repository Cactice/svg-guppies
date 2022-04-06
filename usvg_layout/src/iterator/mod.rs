mod line_to_parallel_lines;
use glam::DVec2;
use line_to_parallel_lines::line_to_parallel_lines;
use usvg::{self, PathData, PathSegment};

pub type Index = u32;
pub fn iterate(path: &PathData, width: f64) -> (Vec<Vertex>, Vec<Index>) {
    let mut vertices: Vec<Vertex> = vec![];
    let mut indices: Vec<Index> = vec![];
    let mut current_vec2: DVec2 = DVec2::new(0.0, 0.0);
    path.iter().for_each(|path| match path {
        PathSegment::MoveTo { x, y } => {
            current_vec2 = DVec2::new(*x, *y);
        }
        PathSegment::LineTo { x, y } => {
            // Below wiki is a reference of what is being done here
            // https://github.com/nical/lyon/wiki/Stroke-tessellation
            let next_vec2 = DVec2::new(*x, *y);
            let ((p0, p1), (p2, p3)) = line_to_parallel_lines((current_vec2, next_vec2), width);
            let new_vertices: Vec<Vertex> = [p0, p1, p2, p3]
                .iter()
                .map(|p| Vertex::from_vec2(p))
                .collect();
            vertices.extend(new_vertices);
            let len = vertices.len() as u32;
            // indices pattern to create two triangles that make a rectangle
            let new_indices: Vec<Index> = [4, 3, 2, 3, 2, 1]
                .iter()
                .map(|index_diff| len - index_diff)
                .collect();
            indices.extend(new_indices);
            current_vec2 = next_vec2;
        }
        PathSegment::CurveTo {
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        } => {
            // TODO: This is not curving at all
            let next_vec2 = DVec2::new(*x, *y);
            let ((p0, p1), (p2, p3)) = line_to_parallel_lines((current_vec2, next_vec2), width);
            let new_vertices: Vec<Vertex> = [p0, p1, p2, p3]
                .iter()
                .map(|p| Vertex::from_vec2(p))
                .collect();
            vertices.extend(new_vertices);
            let len = vertices.len() as u32;
            // indices pattern to create two triangles that make a rectangle
            let new_indices: Vec<Index> = [4, 3, 2, 3, 2, 1]
                .iter()
                .map(|index_diff| len - index_diff)
                .collect();
            indices.extend(new_indices);
            current_vec2 = next_vec2;
        }
        PathSegment::ClosePath => {}
    });
    return (vertices, indices);
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
    fn from_vec2(v: &DVec2) -> Self {
        Self {
            position: [(v.x / 500.0) as f32, (-v.y / 500.0) as f32, 0.0],
            ..Default::default()
        }
    }
}
