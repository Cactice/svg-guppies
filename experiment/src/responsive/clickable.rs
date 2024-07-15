use super::layout_machine::LayoutMachine;
use guppies::glam::Mat4;
use guppies::glam::Vec4;

#[derive(Debug, Clone)]
pub enum ClickableBbox {
    Bbox(Mat4),
    Layout(String),
}

impl ClickableBbox {
    pub fn click_detection(&self, click: Vec4, layout_machine: &LayoutMachine) -> bool {
        let bbox = match self {
            ClickableBbox::Layout(id) => layout_machine.get_bbox_for(id.to_string()).unwrap(),
            ClickableBbox::Bbox(bbox) => *bbox,
        };
        let click = bbox.inverse()
            * Mat4::from_scale([1., -1., 1.].into())
            * Mat4::from_translation([-1.0, -1., 0.].into())
            * layout_machine.display_mat4.inverse()
            * click;
        if click.x.abs() < 1. && click.y.abs() < 1. {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct Clickable {
    pub bbox: ClickableBbox,
    pub id: String,
}
