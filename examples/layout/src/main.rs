mod call_back;
mod constraint;

use bytemuck::cast_slice;
use call_back::{get_x_constraint, get_y_constraint};
use concept::svg_init::get_default_init_callback;
use constraint::Constraint;
use guppies::{glam::Mat4, primitives::Rect, winit::dpi::PhysicalSize};
use mobile_entry_point::mobile_entry_point;
use salvage::{
    callback::PassDown,
    svg_set::SvgSet,
    usvg::{self, NodeExt},
};
use std::vec;

fn get_svg_size(svg_scale: Rect) -> Mat4 {
    Mat4::from_scale([svg_scale.size.x as f32, svg_scale.size.y as f32, 1.].into())
}

fn get_screen_size(size: PhysicalSize<u32>) -> Mat4 {
    Mat4::from_scale([size.width as f32, size.height as f32, 1.].into())
}

fn layout_recursively(svg: Mat4, display: Mat4, node: usvg::Node, parent: Mat4) -> Vec<Mat4> {
    let mut children_transforms: Vec<Mat4> = node
        .children()
        .into_iter()
        .flat_map(|child| layout_recursively(svg, display, child, parent))
        .collect();

    if let Some(bbox) = node.calculate_bbox() {
        let id = node.id();
        let constraint = Constraint {
            x: get_x_constraint(&id),
            y: get_y_constraint(&id),
        };
        if node.id().contains("#transform") {
            children_transforms.insert(0, constraint.to_mat4(display, svg, bbox));
        }
    }
    children_transforms
}

pub fn main() {
    let svg_set = SvgSet::new(
        include_str!("../MenuBar.svg"),
        PassDown {
            transform_id: 1,
            ..Default::default()
        },
        get_default_init_callback(),
    );
    guppies::render_loop(move |event, gpu_redraw| {
        if let guppies::winit::event::Event::WindowEvent {
            event: guppies::winit::event::WindowEvent::Resized(p),
            ..
        } = event
        {
            let display = get_screen_size(*p);
            let svg = get_svg_size(svg_set.bbox);

            let mut transforms = layout_recursively(svg, display, svg_set.root.clone(), svg);
            let mut answer_transforms = vec![Mat4::IDENTITY, Mat4::IDENTITY];
            answer_transforms.append(&mut transforms);
            gpu_redraw.update_texture([cast_slice(&answer_transforms[..])].concat());
            gpu_redraw.update_triangles(svg_set.get_combined_geometries().triangles, 0);
        }
    });
}

#[mobile_entry_point]
pub fn mobile_main() {
    main()
}
