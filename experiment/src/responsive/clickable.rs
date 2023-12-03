use super::constraint::get_normalize_scale;
use super::layout::Layout;
use guppies::glam::Mat4;
use guppies::glam::Vec4;

#[derive(Debug, Clone)]
pub enum ClickableBbox {
    Bbox(Mat4),
    Layout(Layout),
}

impl ClickableBbox {
    pub fn click_detection(&self, click: Vec4, display: Mat4) -> bool {
        let bbox = match self {
            ClickableBbox::Layout(layout) => {
                layout.to_mat4(
                    display,
                    Mat4::IDENTITY, /*FIXME: this is completely wrong, consider how to get parent bbox*/
                ) * layout.bbox
            }
            ClickableBbox::Bbox(bbox) => *bbox,
        };
        let click = Mat4::from_translation([-1., 1., 0.].into())
            * Mat4::from_scale([0.5, 0.5, 1.].into())
            * get_normalize_scale(display, Mat4::IDENTITY)
            * click;
        let click = bbox.inverse() * click;
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
