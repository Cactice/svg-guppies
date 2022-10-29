use crate::{
    rect::{Constraint, MyRect, XConstraint, YConstraint},
    MyPassDown,
};
use concept::svg_init::{regex::RegexSet, RegexPatterns};
use guppies::{glam::Mat4, primitives::Rect, winit::dpi::PhysicalSize};
use salvage::{
    callback::IndicesPriority,
    geometry::Geometry,
    usvg::{self, Node, NodeExt},
};

pub fn get_fullscreen_scale(svg_scale: Rect) -> Mat4 {
    Mat4::from_scale(
        [
            1. / svg_scale.size.x as f32,
            1. / svg_scale.size.y as f32,
            1.,
        ]
        .into(),
    )
}

pub fn get_normalization() -> Mat4 {
    Mat4::from_scale([2., -2., 1.].into()) * Mat4::from_translation([-0.5, -0.5, 1.].into())
}
pub fn get_svg_normalization_1(size: PhysicalSize<u32>) -> Mat4 {
    Mat4::from_scale([1. / size.width as f32, 1. / size.height as f32, 1.].into())
}

pub fn get_svg_normalization(size: PhysicalSize<u32>) -> Mat4 {
    Mat4::from_scale([1. / size.width as f32, 1. / size.height as f32, 1.].into())
}

pub fn get_my_init_callback() -> impl FnMut(Node, MyPassDown) -> (Option<Geometry>, MyPassDown) {
    let mut transform_count = 1;
    let mut regex_patterns = RegexPatterns::default();
    let dynamic = regex_patterns.add(r"#dynamic(?:$| |#)");
    let dynamic_text = regex_patterns.add(r"#dynamicText(?:$| |#)");
    let defaults = RegexSet::new(regex_patterns.inner.iter().map(|r| &r.regex_pattern)).unwrap();
    move |node, pass_down| {
        let id = node.id();
        let default_matches = defaults.matches(&id);
        let MyPassDown {
            transform_id: parent_transform_id,
            indices_priority: parent_priority,
            bbox: parent_bbox,
        } = pass_down;
        let bbox = node.calculate_bbox();
        let x_constraint = if let (Some(parent_bbox), Some(bbox)) = (parent_bbox, bbox) {
            get_x_constraint(&id)
        } else {
            XConstraint::Scale
        };
        let transform_id =
            if default_matches.matched(dynamic.index) || x_constraint != XConstraint::Scale {
                transform_count += 1;
                transform_count
            } else {
                parent_transform_id
            };
        let indices_priority = if !default_matches.matched(dynamic_text.index) {
            IndicesPriority::Variable
        } else {
            IndicesPriority::Fixed
        };
        let indices_priority = parent_priority.max(indices_priority);
        let geometry = {
            if let usvg::NodeKind::Path(ref p) = *node.borrow() {
                Some(Geometry::new(p, transform_id, indices_priority))
            } else {
                None
            }
        };
        (
            geometry,
            MyPassDown {
                indices_priority,
                transform_id,
                bbox,
            },
        )
    }
}

pub fn get_y_constraint(id: &str, bbox: &MyRect, parent_bbox: &MyRect) -> YConstraint {
    let mut regex_patterns = RegexPatterns::default();
    let yt = regex_patterns.add(r"#yt(?:$| |#)");
    let yb = regex_patterns.add(r"#yb(?:$| |#)");
    let ytb = regex_patterns.add(r"#ytb(?:$| |#)");
    let yc = regex_patterns.add(r"#yc(?:$| |#)");
    let constraint_regex =
        RegexSet::new(regex_patterns.inner.iter().map(|r| &r.regex_pattern)).unwrap();
    let matches = constraint_regex.matches(id);
    let top_diff = (parent_bbox.bottom() - bbox.bottom()) as f32;
    let bottom_diff = (parent_bbox.top() - bbox.top()) as f32;
    if matches.matched(yt.index) {
        YConstraint::Top(top_diff)
    } else if matches.matched(yb.index) {
        YConstraint::Bottom(bottom_diff)
    } else if matches.matched(ytb.index) {
        YConstraint::TopAndBottom {
            top: top_diff,
            bottom: bottom_diff,
        }
    } else if matches.matched(yc.index) {
        YConstraint::Center {
            downward_from_center: (parent_bbox.y_center() - parent_bbox.y_center()) as f32,
        }
    } else {
        YConstraint::Scale
    }
}

pub fn get_x_constraint(id: &str) -> XConstraint {
    let mut regex_patterns = RegexPatterns::default();
    let menu = regex_patterns.add(r"Menu #transform");
    let grab = regex_patterns.add(r"Grab #transform");
    let undo = regex_patterns.add(r"Undo #transform");
    let constraint_regex =
        RegexSet::new(regex_patterns.inner.iter().map(|r| &r.regex_pattern)).unwrap();
    let matches = constraint_regex.matches(id);
    if matches.matched(menu.index) {
        XConstraint::Left(15.)
    } else if matches.matched(grab.index) {
        XConstraint::Center(0.)
    } else if matches.matched(undo.index) {
        XConstraint::Right(15.)
    } else {
        XConstraint::Scale
    }
}
