use guppies::glam::Vec2;
pub use regex;
use regex::RegexSet;
use salvage::{
    callback::PassDown,
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
pub const LAYOUT_REGEX: &str = r"#layout(?:$| |#)";
pub const DYNAMIC_TEXT_REGEX: &str = r"#dynamicText(?:$| |#)";

pub fn get_default_init_callback() -> impl FnMut(Node, PassDown) -> (Option<Geometry>, PassDown) {
    let mut transform_count = 1;
    let mut regex_patterns = RegexPatterns::default();
    let _clickable_regex_pattern = regex_patterns.add(CLICKABLE_REGEX);
    let transform_regex_pattern = regex_patterns.add(TRANSFORM_REGEX);
    let _dynamic_text_regex_pattern = regex_patterns.add(DYNAMIC_TEXT_REGEX);
    let defaults = RegexSet::new(regex_patterns.inner.iter().map(|r| &r.regex_pattern)).unwrap();
    move |node, pass_down| {
        let PassDown {
            transform_id: parent_transform_id,
        } = pass_down;
        let id = node.id();
        let default_matches = defaults.matches(&id);
        let transform_id = if default_matches.matched(transform_regex_pattern.index) {
            transform_count += 1;
            transform_count
        } else {
            parent_transform_id
        };
        let geometry = {
            match *node.borrow() {
                usvg::NodeKind::Path(ref p) => Some(Geometry::new(p, transform_id)),
                _ => None,
            }
        };
        (geometry, PassDown { transform_id })
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
