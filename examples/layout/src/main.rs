use bytemuck::cast_slice;
use concept::{scroll::ScrollState, uses::use_svg};
use guppies::glam::{Mat2, Mat4, Vec2};
use salvage::usvg::{Node, NodeExt, PathBbox};

enum HorizontalConstraint {
    Left(f32),
    Right(f32),
    LeftAndRight { left: f32, right: f32 },
    Center { rightward_from_center: f32 },
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

fn recursive_layout(node: Node, parent_box: Mat2, constraint: Constraint) {
    let Vec2 {
        x: parent_x,
        y: parent_width,
    } = parent_box.x_axis;
    let parent_center = parent_x + parent_width / 2.;
    let x_constraint = constraint.x;
    let width = node.calculate_bbox().unwrap().width() as f32;

    let mut this_box = parent_box.clone();
    this_box.x_axis.y = width;
    match x_constraint {
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
    };
}

pub fn main() {
    let svg_set = use_svg(include_str!("../Menu.svg"), |_, _| {});
    let mut scroll_state = ScrollState::new_from_svg_set(&svg_set);
    guppies::render_loop(move |event, gpu_redraw| {
        scroll_state.event_handler(event);
        gpu_redraw.update_triangles(svg_set.get_combined_geometries().triangles, 0);
        gpu_redraw
            .update_texture([cast_slice(&[scroll_state.transform, Mat4::default()])].concat());
    });
}
