use crate::prepare_triangles_from_path::prepare_triangles_from_path;
use guppies::primitives::{Indices, Triangles};
use usvg::{Path, Tree};

#[derive(Clone, Debug, Default)]
pub struct Geometry {
    pub triangles: Triangles,
    pub id: String,
}
impl Geometry {
    pub fn from_tree(tree: Tree, transform_id: u32) -> Self {
        let geometry = tree
            .root()
            .descendants()
            .into_iter()
            .filter_map(|node| {
                if let usvg::NodeKind::Path(ref p) = *node.borrow() {
                    Some(Geometry::new(p, transform_id))
                } else {
                    None
                }
            })
            .fold(Geometry::default(), |acc, curr| acc.extend(&curr));
        geometry
    }
    pub fn extend(mut self, other: &Self) -> Self {
        let v_len = self.triangles.vertices.len() as u32;
        let other_indices_with_offset: Indices =
            other.triangles.indices.iter().map(|i| i + v_len).collect();
        self.triangles
            .vertices
            .extend(other.triangles.vertices.iter());
        self.triangles.indices.extend(other_indices_with_offset);
        self
    }
    pub fn new(p: &Path, transform_id: u32) -> Self {
        let triangles = prepare_triangles_from_path(p, transform_id);
        Self {
            triangles,
            id: p.id.to_owned(),
        }
    }
}
