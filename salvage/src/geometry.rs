use crate::{callback::IndicesPriority, prepare_vertex_buffer::prepare_vertex_buffer};
use guppies::{
    glam::Vec2,
    primitives::{Indices, Rect, Triangles, Vertices},
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
    pub fn extend(&mut self, other: &Self, with_offset: u32) {
        let v_len = self.vertices.len() as u32;
        let other_indices_with_offset: Indices = other
            .indices
            .iter()
            .map(|i| i + v_len + with_offset)
            .collect();
        self.vertices.extend(other.vertices.iter());
        self.indices.extend(other_indices_with_offset);
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
