mod call_back;
mod constraint;

use bytemuck::cast_slice;
use call_back::get_constraint;
use concept::{
    svg_init::{regex::Regex, TRANSFORM_REGEX},
    uses::use_svg,
};
use constraint::Constraint;
use guppies::{
    glam::{Mat4, Vec3},
    primitives::Rect,
    winit::dpi::PhysicalSize,
};
use mobile_entry_point::mobile_entry_point;
use salvage::usvg::{self, NodeExt};
use std::vec;

struct Layout {
    constraint: Constraint,
    bbox: Mat4,
}

fn get_svg_size(svg_scale: Rect) -> Mat4 {
    Mat4::from_scale([svg_scale.size.x as f32, svg_scale.size.y as f32, 1.].into())
}

fn get_screen_size(size: PhysicalSize<u32>) -> Mat4 {
    Mat4::from_scale([size.width as f32, size.height as f32, 1.].into())
}

fn get_layout(node: &usvg::Node) -> Option<Layout> {
    let transform_regex = Regex::new(TRANSFORM_REGEX).unwrap();
    let id = node.id();
    if transform_regex.is_match(&id) {
        if let Some(bbox) = node.calculate_bbox() {
            let bbox_mat4 = Mat4::from_scale_rotation_translation(
                [bbox.width() as f32, bbox.height() as f32, 0.].into(),
                Default::default(),
                [bbox.x() as f32, bbox.y() as f32, 0.].into(),
            );
            let constraint = get_constraint(&id);

            return Some(Layout {
                constraint,
                bbox: bbox_mat4,
            });
        }
    }
    None
}

pub fn main() {
    let mut layouts = Vec::new();
    let mut display_mat4 = Mat4::IDENTITY;
    let mut svg_mat4 = Mat4::IDENTITY;

    let svg_set = use_svg(include_str!("../MenuBar.svg"), |node, _pass_down| {
        if let Some(layout) = get_layout(&node) {
            layouts.push(layout);
        }
    });

    guppies::render_loop(move |event, gpu_redraw| {
        if let guppies::winit::event::Event::WindowEvent { event, .. } = event {
            match event {
                guppies::winit::event::WindowEvent::Resized(p) => {
                    display_mat4 = get_screen_size(*p);
                    svg_mat4 = get_svg_size(svg_set.bbox);
                    let mut transforms = vec![Mat4::IDENTITY, Mat4::IDENTITY];
                    transforms.append(
                        &mut layouts
                            .iter()
                            .map(|layout| {
                                layout
                                    .constraint
                                    .to_mat4(display_mat4, svg_mat4, layout.bbox)
                            })
                            .collect(),
                    );
                    gpu_redraw.update_texture([cast_slice(&transforms[..])].concat());
                    gpu_redraw.update_triangles(svg_set.get_combined_geometries().triangles, 0);
                }

                guppies::winit::event::WindowEvent::Touch(touch) => {
                    let touch = Vec3::from([touch.location.x as f32, touch.location.y as f32, 0.]);
                    let touch_is_on_node = todo!();
                }
                _ => {}
            }
        }
    });
}

#[mobile_entry_point]
pub fn mobile_main() {
    main()
}
