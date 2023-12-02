use super::constraint::{XConstraint, YConstraint};
use guppies::glam::{Mat4, Vec3};

pub(crate) enum CommonConstraint {
    Start(f32),
    End(f32),
    StartAndEnd { start: f32, end: f32 },
    Center(f32),
    Scale,
}

impl From<XConstraint> for CommonConstraint {
    fn from(x_constraint: XConstraint) -> Self {
        match x_constraint {
            XConstraint::Left(left) => CommonConstraint::Start(left),
            XConstraint::Right(right) => CommonConstraint::End(right),
            XConstraint::LeftAndRight { left, right } => CommonConstraint::StartAndEnd {
                start: left,
                end: right,
            },
            XConstraint::Center(x) => CommonConstraint::Center(x),
            XConstraint::Scale => CommonConstraint::Scale,
        }
    }
}
impl From<YConstraint> for CommonConstraint {
    fn from(y_constraint: YConstraint) -> Self {
        match y_constraint {
            YConstraint::Top(top) => CommonConstraint::End(top),
            YConstraint::Bottom(bottom) => CommonConstraint::Start(bottom),
            YConstraint::TopAndBottom { top, bottom } => CommonConstraint::StartAndEnd {
                start: bottom,
                end: top,
            },
            YConstraint::Center(y) => CommonConstraint::Center(y),
            YConstraint::Scale => CommonConstraint::Scale,
        }
    }
}
impl CommonConstraint {
    pub(crate) fn to_pre_post_transform<F: Fn(Vec3) -> f32, G: Fn(f32, f32) -> Vec3>(
        self,
        display: Mat4,
        bbox: Mat4,
        parent_bbox: Mat4,
        accessor: F,
        composer: G,
    ) -> (Mat4, Mat4) {
        let fill = Mat4::from_scale(composer(
            accessor(display.to_scale_rotation_translation().0 / 2.)
                / accessor(bbox.to_scale_rotation_translation().0),
            1.0,
        ));

        let (left_align, right_align, center) = prepare_anchor_points(bbox, &accessor, &composer);
        // let (parent_fill, parent_left_align, right_align, center) =
        // prepare_anchor_points(parent_bbox, display, &accessor, &composer);

        let pre_normalize_transform;
        let post_normalize_transform;
        match self {
            CommonConstraint::Start(left) => {
                pre_normalize_transform = left_align * Mat4::from_translation(composer(left, 0.));
                post_normalize_transform = Mat4::from_translation(composer(-0.5, 0.));
            }
            CommonConstraint::End(right) => {
                pre_normalize_transform = right_align * Mat4::from_translation(composer(right, 0.));
                post_normalize_transform = Mat4::from_translation(composer(0.5, 0.));
            }
            CommonConstraint::Center(rightward_from_center) => {
                pre_normalize_transform =
                    center * Mat4::from_translation(composer(rightward_from_center, 0.));
                post_normalize_transform = Mat4::IDENTITY;
            }
            CommonConstraint::StartAndEnd { start, end } => {
                todo!();
            }
            CommonConstraint::Scale => {
                pre_normalize_transform = fill * center;
                post_normalize_transform = Mat4::IDENTITY;
            }
        };
        (pre_normalize_transform, post_normalize_transform)
    }
}

fn prepare_anchor_points<F: Fn(Vec3) -> f32, G: Fn(f32, f32) -> Vec3>(
    bbox: Mat4,
    accessor: &F,
    composer: &G,
) -> (Mat4, Mat4, Mat4) {
    let (bbox_scale, _, bbox_translation) = bbox.to_scale_rotation_translation();

    let start_align = Mat4::from_translation(composer(accessor(bbox_translation), 0.)).inverse();
    let end_align = Mat4::from_translation(composer(
        accessor(bbox_translation) + accessor(bbox_scale),
        0.,
    ))
    .inverse();
    let center = Mat4::from_translation(composer(
        accessor(bbox_translation) + accessor(bbox_scale) / 2.,
        0.,
    ))
    .inverse();
    (start_align, end_align, center)
}
