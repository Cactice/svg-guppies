mod call_back;
mod rect;
use bytemuck::cast_slice;
use call_back::{get_constraint, get_my_init_callback};
use concept::{scroll::ScrollState, svg_init::get_default_init_callback};
use guppies::glam::Mat4;
use rect::{MyRect, XConstraint};
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
fn layout_recursively(node: &Node, parent_bbox: MyRect, transforms: &mut Mat4) {
    let bbox = node.calculate_bbox();
    if let Some(bbox) = bbox {
        let mut bbox = MyRect::from(bbox);
        let constraint_x = get_constraint(&node.id(), &bbox, &parent_bbox);

        match constraint_x {
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
