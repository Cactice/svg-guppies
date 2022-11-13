use guppies::glam::Vec2;
pub use regex;
use regex::RegexSet;
use salvage::{
    callback::{IndicesPriority, PassDown},
    geometry::Geometry,
    usvg::{self, Node, NodeExt},
};
#[derive(Clone, Debug, Default)]
pub struct RegexPattern {
    pub regex_pattern: String,
    pub index: usize,
}
#[derive(Clone, Debug, Default)]
pub struct RegexPatterns {
    pub inner: Vec<RegexPattern>,
}

impl RegexPatterns {
    pub fn add(&mut self, regex_pattern: &str) -> RegexPattern {
        let regex_pattern = RegexPattern {
            regex_pattern: regex_pattern.to_string(),
            index: self.inner.len(),
        };
        self.inner.push(regex_pattern.clone());
        regex_pattern
    }
}

pub const CLICKABLE_REGEX: &str = r"#clickable(?:$| |#)";
pub const TRANSFORM_REGEX: &str = r"#transform(?:$| |#)";
pub const DYNAMIC_TEXT_REGEX: &str = r"#dynamicText(?:$| |#)";

pub fn get_default_init_callback() -> impl FnMut(Node, PassDown) -> (Option<Geometry>, PassDown) {
    let mut transform_count = 1;
    let mut regex_patterns = RegexPatterns::default();
    let _clickable_regex_pattern = regex_patterns.add(CLICKABLE_REGEX);
    let transform_regex_pattern = regex_patterns.add(TRANSFORM_REGEX);
    let dynamic_text_regex_pattern = regex_patterns.add(DYNAMIC_TEXT_REGEX);
    let defaults = RegexSet::new(regex_patterns.inner.iter().map(|r| &r.regex_pattern)).unwrap();
    move |node, pass_down| {
        let PassDown {
            transform_id: parent_transform_id,
            indices_priority: parent_priority,
        } = pass_down;
        let id = node.id();
        let default_matches = defaults.matches(&id);
        let transform_id = if default_matches.matched(transform_regex_pattern.index) {
            transform_count += 1;
            transform_count
        } else {
            parent_transform_id
        };
        let indices_priority = if !default_matches.matched(dynamic_text_regex_pattern.index) {
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
            PassDown {
                indices_priority,
                transform_id,
            },
        )
    }
}

pub fn get_center(node: &Node) -> Vec2 {
    let bbox = node.calculate_bbox().unwrap();
    let center = Vec2::new(
        (bbox.x() + bbox.width() / 2.) as f32,
        (bbox.y() + bbox.height() / 2.) as f32,
    );
    center
}
