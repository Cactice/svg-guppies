use crate::{callback::IndicesPriority, prepare_vertex_buffer::prepare_vertex_buffer};
use guppies::{
    glam::Vec2,
    primitives::{Indices, Rect, Vertices},
};
use usvg::{Path, PathBbox};
fn rect_from_bbox(bbox: &PathBbox) -> Rect {
    Rect {
        position: Vec2::new(bbox.x() as f32, bbox.y() as f32),
        size: Vec2::new(bbox.width() as f32, bbox.height() as f32),
    }
}

#[derive(Clone, Debug, Default)]
pub struct Geometry {
    vertices: Vertices,
    indices: Indices,
    priority: IndicesPriority,
}
impl Geometry {
    pub fn get_vertices_len(&self) -> usize {
        self.vertices.len()
    }
    pub fn get_v(&self) -> Vertices {
        self.vertices.clone()
    }
    pub fn get_i_with_offset(&self, offset: u32) -> Indices {
        self.indices.iter().map(|index| index + offset).collect()
    }
    pub fn new(p: &Path, transform_id: u32, priority: IndicesPriority) -> Self {
        let v = prepare_vertex_buffer(p, transform_id);
        Self {
            vertices: v.vertices,
            indices: v.indices,
            priority,
        }
    }
}
