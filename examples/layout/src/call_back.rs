use concept::svg_init::{regex::RegexSet, RegexPatterns};
use salvage::{
    callback::IndicesPriority,
    geometry::Geometry,
    usvg::{self, Node, NodeExt},
};

use crate::{rect::XConstraint, MyPassDown};

pub fn get_my_init_callback() -> impl FnMut(Node, MyPassDown) -> (Option<Geometry>, MyPassDown) {
    let mut transform_count = 1;
    let mut regex_patterns = RegexPatterns::default();
    let hr = regex_patterns.add(r"#hr(?:$| |#)");
    let hl = regex_patterns.add(r"#hl(?:$| |#)");
    let hlr = regex_patterns.add(r"#hlr(?:$| |#)");
    let hc = regex_patterns.add(r"#hc(?:$| |#)");
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
        if let (Some(parent_bbox), Some(bbox)) = (parent_bbox, bbox) {
            dbg!(node.id());
            let right_diff = (parent_bbox.right() - bbox.right()) as f32;
            let left_diff = (parent_bbox.left() - bbox.left()) as f32;
            let constraint_x = if default_matches.matched(hr.index) {
                XConstraint::Right(right_diff)
            } else if default_matches.matched(hl.index) {
                XConstraint::Left(left_diff)
            } else if default_matches.matched(hlr.index) {
                XConstraint::LeftAndRight {
                    left: left_diff,
                    right: right_diff,
                }
            } else if default_matches.matched(hc.index) {
                let parent_center = parent_bbox.left() + (parent_bbox.width() / 2.);
                let center = bbox.left() + (bbox.width() / 2.);
                XConstraint::Center {
                    rightward_from_center: (parent_center - center) as f32,
                }
            } else {
                XConstraint::Scale
            };
        };
        let transform_id = if default_matches.matched(dynamic.index) {
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
