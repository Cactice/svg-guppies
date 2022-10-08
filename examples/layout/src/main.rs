use bytemuck::cast_slice;
use concept::{
    scroll::ScrollState,
    svg_init::{get_default_init_callback, regex::RegexSet, RegexPatterns},
};
use guppies::glam::{Mat2, Mat4};
use salvage::{
    callback::{IndicesPriority, PassDown},
    svg_set::SvgSet,
    usvg::{Node, NodeExt, PathBbox},
};

#[derive(Clone, Default)]
pub struct MyPassDown {
    pub indices_priority: IndicesPriority,
    pub transform_id: u32,
    pub bbox: Option<PathBbox>,
}
#[derive(Copy, Clone, Default, Debug)]
pub struct MyRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}
impl MyRect {
    fn right(&mut self) -> f32 {
        self.x + self.width
    }
    fn left(&mut self) -> f32 {
        self.x
    }
    fn x_center(&mut self) -> f32 {
        self.x + (self.width / 2.)
    }
}
impl From<PathBbox> for MyRect {
    fn from(bbox: PathBbox) -> Self {
        Self {
            x: bbox.x() as f32,
            y: bbox.y() as f32,
            width: bbox.width() as f32,
            height: bbox.height() as f32,
        }
    }
}

enum XConstraint {
    Left(f32),
    Right(f32),
    LeftAndRight { left: f32, right: f32 },
    Center { rightward_from_center: f32 },
    Scale,
}

impl Default for XConstraint {
    fn default() -> Self {
        Self::LeftAndRight {
            left: 0.,
            right: 0.,
        }
    }
}
enum YConstraint {
    Top(f32),
    Bottom(f32),
    TopAndBottom { top: f32, bottom: f32 },
    Center { downward_from_center: f32 },
}

impl Default for YConstraint {
    fn default() -> Self {
        Self::TopAndBottom {
            top: 0.,
            bottom: 0.,
        }
    }
}

struct Constraint {
    x: XConstraint,
    y: YConstraint,
}

fn layout_recursively(node: &Node, parent_bbox: Option<MyRect>, constraint: Constraint) {
    let id = node.id();
    // letMyRect {
    //     x: parent_x,
    //     width: parent_width,
    //     ..
    // } = parent_bbox;

    let mut regex_patterns = RegexPatterns::default();
    let xr = regex_patterns.add(r"#xr(?:$| |#)");
    let xl = regex_patterns.add(r"#xl(?:$| |#)");
    let xlr = regex_patterns.add(r"#xlr(?:$| |#)");
    let xc = regex_patterns.add(r"#xc(?:$| |#)");
    let defaults = RegexSet::new(regex_patterns.inner.iter().map(|r| &r.regex_pattern)).unwrap();

    let default_matches = defaults.matches(&id);
    let bbox = node.calculate_bbox();
    if let (Some(mut parent_bbox), Some(bbox)) = (parent_bbox, bbox) {
        let mut bbox = MyRect::from(bbox);
        let right_diff = (parent_bbox.right() - bbox.right()) as f32;
        let left_diff = (parent_bbox.left() - bbox.left()) as f32;
        let constraint_x = if default_matches.matched(xr.index) {
            XConstraint::Right(right_diff)
        } else if default_matches.matched(xl.index) {
            XConstraint::Left(left_diff)
        } else if default_matches.matched(xlr.index) {
            XConstraint::LeftAndRight {
                left: left_diff,
                right: right_diff,
            }
        } else if default_matches.matched(xc.index) {
            XConstraint::Center {
                rightward_from_center: (parent_bbox.x_center() - parent_bbox.x_center()) as f32,
            }
        } else {
            XConstraint::Scale
        };

        match constraint.x {
            XConstraint::Left(left) => bbox.x += left,
            XConstraint::Right(right) => bbox.x += parent_bbox.width - (right + bbox.width),
            XConstraint::LeftAndRight { left, right } => {
                bbox.width = bbox.width - (left + right);
                bbox.x += left;
            }
            XConstraint::Center {
                rightward_from_center,
            } => {
                bbox.x = parent_bbox.x_center() + rightward_from_center;
            }
            XConstraint::Scale => {}
        };
    };
}

pub fn main() {
    let svg_set = SvgSet::new(
        include_str!("../Menu.svg"),
        PassDown {
            transform_id: 1,
            ..Default::default()
        },
        get_default_init_callback(),
    );
    let mut scroll_state = ScrollState::new_from_svg_set(&svg_set);
    guppies::render_loop(move |event, gpu_redraw| {
        scroll_state.event_handler(event);
        gpu_redraw.update_triangles(svg_set.get_combined_geometries().triangles, 0);
        gpu_redraw
            .update_texture([cast_slice(&[scroll_state.transform, Mat4::default()])].concat());
    });
}
