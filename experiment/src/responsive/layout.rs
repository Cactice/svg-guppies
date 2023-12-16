use super::constraint::Constraint;
use guppies::glam::Mat4;
use guppies::winit::dpi::PhysicalSize;
use html_escape::decode_html_entities;
use regex::Regex;
use salvage::usvg::{self};
use salvage::usvg::{NodeExt, PathBbox};

pub(crate) fn size_to_mat4(size: PhysicalSize<u32>) -> Mat4 {
    Mat4::from_scale([size.width as f32, size.height as f32, 1.].into())
}

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub constraint: Constraint,
    pub bbox: Mat4,
}

impl Layout {
    pub fn to_mat4(self, display: Mat4, parent_bbox: Mat4) -> Mat4 {
        self.constraint.to_mat4(display, self.bbox, parent_bbox)
    }
    pub fn new(node: &usvg::Node) -> Self {
        let id = node.id();
        let re = Regex::new(r"#layout (.+)").unwrap();
        let json = &re.captures(&id).unwrap()[1];
        let json = decode_html_entities(json).to_string();
        let constraint = serde_json::from_str::<Constraint>(&json).unwrap();
        let bbox_mat4 = bbox_to_mat4(
            node.calculate_bbox()
                .expect("Elements with #transform should be able to calculate bbox"),
        );
        return Layout {
            constraint,
            bbox: bbox_mat4,
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
