use guppies::glam::Vec2;
pub use regex;
use regex::RegexSet;
use salvage::{
    geometry::Geometry,
    usvg::{self, Node, NodeExt},
};

use crate::responsive::layout::Layout;
#[derive(Clone, Debug)]
pub struct PassDown {
    pub transform_id: u32,
    pub is_include: bool,
    pub parent_layouts: Vec<Layout>,
}

impl Default for PassDown {
    fn default() -> Self {
        Self {
            transform_id: 1,
            is_include: true,
            parent_layouts: [].to_vec(),
        }
    }
}

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
pub const COMPONENT_REGEX: &str = r"#component(?:$| |#)";
pub const LAYOUT_REGEX: &str = r"#layout(?:$| |#)";
pub const DYNAMIC_TEXT_REGEX: &str = r"#dynamicText(?:$| |#)";

pub fn get_default_init_callback(
    mut transform_count: u32,
    include: Option<String>,
) -> impl FnMut(Node, PassDown) -> (Option<Geometry>, PassDown) {
    let mut regex_patterns = RegexPatterns::default();
    let _clickable_regex_pattern = regex_patterns.add(CLICKABLE_REGEX);
    let transform_regex_pattern = regex_patterns.add(TRANSFORM_REGEX);
    let include_regex_pattern =
        regex_patterns.add(&include.clone().unwrap_or("xxxxxxxxx".to_string()));
    let component_regex_pattern = regex_patterns.add(COMPONENT_REGEX);
    let _dynamic_text_regex_pattern = regex_patterns.add(DYNAMIC_TEXT_REGEX);
    let defaults = RegexSet::new(regex_patterns.inner.iter().map(|r| &r.regex_pattern)).unwrap();
    move |node, pass_down| {
        let PassDown {
            transform_id: parent_transform_id,
            is_include: parent_is_include,
            parent_layouts,
        } = pass_down;
        let id = node.id();
        let default_matches = defaults.matches(&id);
        let component_matched = default_matches.matched(component_regex_pattern.index);
        let include_matched = default_matches.matched(include_regex_pattern.index);
        let is_include = match parent_is_include && !component_matched {
            true => true,
            false => include_matched,
        };
        let transform_id = match default_matches.matched(transform_regex_pattern.index) {
            true => {
                transform_count += 1;
                transform_count
            }
            false => parent_transform_id,
        };
        let geometry = match is_include {
            true => match *node.borrow() {
                usvg::NodeKind::Path(ref p) => Some(Geometry::new(p, transform_id)),
                _ => None,
            },
            false => None,
        };

        (
            geometry,
            PassDown {
                transform_id,
                is_include,
                parent_layouts,
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
