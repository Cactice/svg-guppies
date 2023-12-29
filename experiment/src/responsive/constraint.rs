use super::common_constraint::CommonConstraint;
use guppies::glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

pub fn get_normalize_scale(display: Mat4) -> Mat4 {
    display.inverse()
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
    pub(crate) fn to_transform(self, bbox: Mat4, parent_bbox: Mat4) -> Mat4 {
        let accessor = |Vec3 { x, .. }| x;
        let composer = |x, other| Vec3 {
            x,
            y: other,
            z: other,
        };
        CommonConstraint::from(self).to_transform(bbox, parent_bbox, accessor, composer)
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
    pub(crate) fn to_transform(self, bbox: Mat4, parent_bbox: Mat4) -> Mat4 {
        let accessor = |Vec3 { y, .. }| y;
        let composer = |y, other| Vec3 {
            x: other,
            y,
            z: other,
        };
        CommonConstraint::from(self).to_transform(bbox, parent_bbox, accessor, composer)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Constraint {
    pub x: XConstraint,
    pub y: YConstraint,
}
impl Default for Constraint {
    fn default() -> Self {
        Self {
            x: XConstraint::Scale,
            y: YConstraint::Scale,
        }
    }
}

impl Constraint {
    pub fn to_mat4(self, display: Mat4, bbox: Mat4, parent_bbox: Mat4) -> Mat4 {
        let Constraint {
            x: constraint_x,
            y: constraint_y,
        } = self;

        let x = constraint_x.to_transform(bbox, parent_bbox);
        let y = constraint_y.to_transform(bbox, parent_bbox);

        return display.inverse() * x * y;
    }
}
