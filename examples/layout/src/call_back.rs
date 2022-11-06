use crate::rect::{MyRect, XConstraint, YConstraint};
use concept::svg_init::{regex::RegexSet, RegexPatterns};
use guppies::{glam::Mat4, primitives::Rect, winit::dpi::PhysicalSize};

pub fn get_svg_size(svg_scale: Rect) -> Mat4 {
    Mat4::from_scale([svg_scale.size.x as f32, svg_scale.size.y as f32, 1.].into())
}

pub fn get_screen_size(size: PhysicalSize<u32>) -> Mat4 {
    Mat4::from_scale([size.width as f32, size.height as f32, 1.].into())
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
        XConstraint::Right(-15.)
    } else {
        XConstraint::Scale
    }
}
