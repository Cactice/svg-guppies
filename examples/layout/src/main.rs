use bytemuck::cast_slice;
use concept::{
    scroll::ScrollState,
    svg_init::{regex::RegexSet, RegexPatterns},
};
use guppies::glam::{Mat2, Mat4, Vec2};
use salvage::{
    callback::IndicesPriority,
    geometry::Geometry,
    svg_set::SvgSet,
    usvg::{self, Node, NodeExt, PathBbox},
};

#[derive(Clone, Default)]
pub struct MyPassDown {
    pub indices_priority: IndicesPriority,
    pub transform_id: u32,
    pub bbox: Option<PathBbox>,
}

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
                HorizontalConstraint::Right(right_diff)
            } else if default_matches.matched(hl.index) {
                HorizontalConstraint::Left(left_diff)
            } else if default_matches.matched(hlr.index) {
                HorizontalConstraint::LeftAndRight {
                    left: left_diff,
                    right: right_diff,
                }
            } else if default_matches.matched(hc.index) {
                let parent_center = parent_bbox.left() + (parent_bbox.width() / 2.);
                let center = bbox.left() + (bbox.width() / 2.);
                HorizontalConstraint::Center {
                    rightward_from_center: (parent_center - center) as f32,
                }
            } else {
                HorizontalConstraint::Scale
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

enum HorizontalConstraint {
    Left(f32),
    Right(f32),
    LeftAndRight { left: f32, right: f32 },
    Center { rightward_from_center: f32 },
    Scale,
}

impl Default for HorizontalConstraint {
    fn default() -> Self {
        Self::LeftAndRight {
            left: 0.,
            right: 0.,
        }
    }
}
enum VerticalConstraint {
    Top(f32),
    Bottom(f32),
    TopAndBottom { top: f32, bottom: f32 },
    Center { downward_from_center: f32 },
}
impl Default for VerticalConstraint {
    fn default() -> Self {
        Self::TopAndBottom {
            top: 0.,
            bottom: 0.,
        }
    }
}

struct Constraint {
    x: HorizontalConstraint,
    y: VerticalConstraint,
}
impl Constraint {
    fn new_from_node() {}
}

fn make_layout(node: &Node, parent_box: Mat2, constraint: Constraint) -> Mat2 {
    let Vec2 {
        x: parent_x,
        y: parent_width,
    } = parent_box.x_axis;
    let parent_center = parent_x + parent_width / 2.;
    let width = node.calculate_bbox().unwrap().width() as f32;

    let mut this_box = parent_box.clone();
    this_box.x_axis.y = width;
    match constraint.x {
        HorizontalConstraint::Left(left) => this_box.x_axis.x += left,
        HorizontalConstraint::Right(right) => this_box.x_axis.x += parent_width - (right + width),
        HorizontalConstraint::LeftAndRight { left, right } => {
            this_box.x_axis.y = this_box.x_axis.y - (left + right);
            this_box.x_axis.x += left;
        }
        HorizontalConstraint::Center {
            rightward_from_center,
        } => {
            this_box.x_axis.x = parent_center + rightward_from_center;
        }
        HorizontalConstraint::Scale => {}
    };
    this_box
}

pub fn main() {
    let svg_set = SvgSet::new(
        include_str!("../Menu.svg"),
        MyPassDown {
            transform_id: 1,
            ..Default::default()
        },
        get_my_init_callback(),
    );
    let mut scroll_state = ScrollState::new_from_svg_set(&svg_set);
    guppies::render_loop(move |event, gpu_redraw| {
        scroll_state.event_handler(event);
        gpu_redraw.update_triangles(svg_set.get_combined_geometries().triangles, 0);
        gpu_redraw
            .update_texture([cast_slice(&[scroll_state.transform, Mat4::default()])].concat());
    });
}
