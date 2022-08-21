use crate::{callback::IndicesPriority, prepare_triangles_from_path::prepare_triangles_from_path};
use guppies::{
    glam::Vec2,
    primitives::{Indices, Rect, Triangles},
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
    pub triangles: Triangles,
    priority: IndicesPriority,
}
impl Geometry {
    pub fn extend(&mut self, other: &Self) {
        let v_len = self.triangles.vertices.len() as u32;
        let other_indices_with_offset: Indices =
            other.triangles.indices.iter().map(|i| i + v_len).collect();
        self.triangles
            .vertices
            .extend(other.triangles.vertices.iter());
        self.triangles.indices.extend(other_indices_with_offset);
    }
    pub fn new(p: &Path, transform_id: u32, priority: IndicesPriority) -> Self {
        let triangles = prepare_triangles_from_path(p, transform_id);
        Self {
            triangles,
            priority,
        }
    }
}
