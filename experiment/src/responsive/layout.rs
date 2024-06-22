use super::constraint::Constraint;
use super::layout_machine::ConstraintMap;
use guppies::glam::Mat4;
use guppies::winit::dpi::PhysicalSize;
use salvage::usvg::{self};
use salvage::usvg::{NodeExt, PathBbox};

pub(crate) fn size_to_mat4(size: PhysicalSize<u32>) -> Mat4 {
    Mat4::from_scale([size.width as f32, size.height as f32, 1.].into())
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub constraint: Constraint,
    pub bbox: Mat4,
    pub parent: Option<String>,
}

impl Layout {
    pub fn to_mat4(&self, display: Mat4, parent_bbox: Mat4) -> Mat4 {
        self.constraint.to_mat4(display, self.bbox, parent_bbox)
    }
    pub fn new(node: &usvg::Node, constraint: Constraint) -> Self {
        if node.id().contains("dynamic") {
            dbg!(node.id(), node.calculate_bbox());
        }
        let bbox_mat4 = bbox_to_mat4(
            node.calculate_bbox()
                .expect("Elements with #transform should be able to calculate bbox"),
        );
        return Layout {
            constraint,
            bbox: bbox_mat4,
            parent: None,
        };
    }
}

pub fn bbox_to_mat4(bbox: PathBbox) -> Mat4 {
    Mat4::from_scale_rotation_translation(
        [bbox.width() as f32, bbox.height() as f32, 1.].into(),
        Default::default(),
        [bbox.x() as f32, bbox.y() as f32, 0.].into(),
    )
}
