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
            YConstraint::Top(top) => CommonConstraint::Start(top),
            YConstraint::Bottom(bottom) => CommonConstraint::End(bottom),
            YConstraint::TopAndBottom { top, bottom } => CommonConstraint::StartAndEnd {
                start: top,
                end: bottom,
            },
            YConstraint::Center(y) => CommonConstraint::Center(y),
            YConstraint::Scale => CommonConstraint::Scale,
        }
    }
}
impl CommonConstraint {
    pub(crate) fn to_transform<F: Fn(Vec3) -> f32, G: Fn(f32, f32) -> Vec3>(
        self,
        bbox: Mat4,
        parent_bbox: Mat4,
        accessor: F,
        composer: G,
    ) -> Mat4 {
        let compose_translation = |number| Mat4::from_translation(composer(number, 0.));
        let compose_scale = |number| Mat4::from_scale(composer(number, 1.));
        let access_scale = |mat4: Mat4| accessor(mat4.to_scale_rotation_translation().0);

        let fill = Mat4::from_scale(composer(
            access_scale(parent_bbox) / access_scale(bbox),
            1.0,
        ));

        let (start, end, center) = prepare_anchor_points(bbox, &accessor, &composer);
        let (start_align, end_align, center_align) =
            (start.inverse(), end.inverse(), center.inverse());
        let (parent_edge_start, parent_edge_end, parent_center) =
            prepare_anchor_points(parent_bbox, &accessor, &composer);

        match self {
            CommonConstraint::Start(start) => {
                compose_translation(start) * parent_edge_start * start_align
            }
            CommonConstraint::End(end) => compose_translation(end) * parent_edge_end * end_align,
            CommonConstraint::Center(towards_end_from_center) => {
                compose_translation(towards_end_from_center) * parent_center * center_align
            }
            CommonConstraint::StartAndEnd { start, end } => {
                let fill_partial =
                    compose_scale((access_scale(parent_bbox) - (start - end)) / access_scale(bbox));
                compose_translation(start) * parent_edge_start * fill_partial * start_align
            }
            CommonConstraint::Scale => parent_center * fill * center_align,
        }
    }
}

fn prepare_anchor_points<F: Fn(Vec3) -> f32, G: Fn(f32, f32) -> Vec3>(
    bbox: Mat4,
    accessor: &F,
    composer: &G,
) -> (Mat4, Mat4, Mat4) {
    let compose_translation = |number| Mat4::from_translation(composer(number, 0.));
    let (bbox_scale, _, bbox_translation) = bbox.to_scale_rotation_translation();

    let start = compose_translation(accessor(bbox_translation));
    let end = compose_translation(accessor(bbox_translation) + accessor(bbox_scale));
    let center = compose_translation(accessor(bbox_translation) + accessor(bbox_scale) / 2.);

    (start, end, center)
}
