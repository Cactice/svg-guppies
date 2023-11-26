use super::common_constraint::CommonConstraint;
use guppies::glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

pub fn get_normalize_scale(display: Mat4) -> Mat4 {
    // Y is flipped because the y axis is in different directions in GPU vs SVG
    // doubling is necessary because GPU range -1 ~ 1 while I used range 0 ~ 1
    // Why last doubling is necessary only god knows.
    // I added it because it looked too small in comparison to figma's prototyping feature.
    Mat4::from_scale([4., -4., 1.].into()) * display.inverse()
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum XConstraint {
    Left(f32),
    Right(f32),
    LeftAndRight { left: f32, right: f32 },
    Center(f32), //rightward_from_center
    Scale,
}

impl Default for XConstraint {
    fn default() -> Self {
        Self::LeftAndRight {
            left: 0.,
            right: 0.,
        }
    }
}

impl XConstraint {
    pub(crate) fn to_pre_post_transform(self, display: Mat4, bbox: Mat4) -> (Mat4, Mat4) {
        let accessor = |Vec3 { x, .. }| x;
        let composer = |x, other| Vec3 {
            x,
            y: other,
            z: other,
        };
        CommonConstraint::from(self).to_pre_post_transform(display, bbox, accessor, composer)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum YConstraint {
    Top(f32),
    Bottom(f32),
    TopAndBottom { top: f32, bottom: f32 },
    Center(f32), //downward_from_center
    Scale,
}

impl Default for YConstraint {
    fn default() -> Self {
        Self::TopAndBottom {
            top: 0.,
            bottom: 0.,
        }
    }
}

impl YConstraint {
    pub(crate) fn to_pre_post_transform(self, display: Mat4, bbox: Mat4) -> (Mat4, Mat4) {
        let accessor = |Vec3 { y, .. }| y;
        let composer = |y, other| Vec3 {
            x: other,
            y,
            z: other,
        };
        CommonConstraint::from(self).to_pre_post_transform(display, bbox, accessor, composer)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Constraint {
    pub x: XConstraint,
    pub y: YConstraint,
}

impl Constraint {
    pub fn to_mat4(self, display: Mat4, bbox: Mat4) -> Mat4 {
        let Constraint {
            x: constraint_x,
            y: constraint_y,
        } = self;

        let (pre_x, post_x) = constraint_x.to_pre_post_transform(display, bbox);
        let (pre_y, post_y) = constraint_y.to_pre_post_transform(display, bbox);

        let pre_xy = pre_x * pre_y;
        let post_xy = post_x * post_y;

        let normalize_scale = get_normalize_scale(display);

        return post_xy * normalize_scale * pre_xy;
    }
}
