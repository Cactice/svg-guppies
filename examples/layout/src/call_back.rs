use crate::{
    rect::{MyRect, XConstraint},
    MyPassDown,
};
use concept::svg_init::{regex::RegexSet, RegexPatterns};
use guppies::{glam::Mat4, winit::dpi::PhysicalSize};
use salvage::{
    callback::IndicesPriority,
    geometry::Geometry,
    usvg::{self, Node, NodeExt},
};

pub fn get_fullscreen_scale(svg_scale: MyRect) -> Mat4 {
    Mat4::from_scale([2. / svg_scale.x as f32, -2. / svg_scale.y as f32, 1.0].into())
}
pub fn get_normalization_scale(size: PhysicalSize<u32>) -> Mat4 {
    Mat4::from_scale([2.0 / size.width as f32, -2.0 / size.height as f32, 1.0].into())
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
            get_constraint(&id, &bbox.into(), &parent_bbox.into())
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

pub fn get_constraint(id: &str, bbox: &MyRect, parent_bbox: &MyRect) -> XConstraint {
    let mut regex_patterns = RegexPatterns::default();
    let constraint_regex =
        RegexSet::new(regex_patterns.inner.iter().map(|r| &r.regex_pattern)).unwrap();
    let matches = constraint_regex.matches(id);
    let xl = regex_patterns.add(r"#xl(?:$| |#)");
    let xr = regex_patterns.add(r"#xr(?:$| |#)");
    let xlr = regex_patterns.add(r"#xlr(?:$| |#)");
    let xc = regex_patterns.add(r"#xc(?:$| |#)");
    let right_diff = (parent_bbox.right() - bbox.right()) as f32;
    let left_diff = (parent_bbox.left() - bbox.left()) as f32;
    let constraint_x = if matches.matched(xr.index) {
        XConstraint::Right(right_diff)
    } else if matches.matched(xl.index) {
        XConstraint::Left(left_diff)
    } else if matches.matched(xlr.index) {
        XConstraint::LeftAndRight {
            left: left_diff,
            right: right_diff,
        }
    } else if matches.matched(xc.index) {
        XConstraint::Center {
            rightward_from_center: (parent_bbox.x_center() - parent_bbox.x_center()) as f32,
        }
    } else {
        XConstraint::Scale
    };
    constraint_x
}
