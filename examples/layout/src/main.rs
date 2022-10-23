mod call_back;
mod rect;
use bytemuck::cast_slice;
use call_back::{get_my_init_callback, get_normalization, get_svg_normalization, get_x_constraint};
use guppies::{
    glam::{Mat4, Vec2},
    primitives::Rect,
};
use rect::{Constraint, MyRect, XConstraint, YConstraint};
use salvage::{
    callback::IndicesPriority,
    svg_set::SvgSet,
    usvg::{Node, NodeExt, PathBbox},
};

#[derive(Clone, Default)]
pub struct MyPassDown {
    pub indices_priority: IndicesPriority,
    pub transform_id: u32,
    pub bbox: Option<PathBbox>,
}
fn layout_recursively(node: &Node, parent_bbox: MyRect, transforms: &mut Vec<Mat4>) {
    let bbox = node.calculate_bbox();
    if let Some(bbox) = bbox {
        let original_bbox = MyRect::from(bbox);
        let mut bbox = MyRect::from(bbox);
        let constraint_x = get_x_constraint(&node.id());

        match constraint_x {
            XConstraint::Left(left) => {
                bbox.x = parent_bbox.x + left;
                transforms.push(Mat4::from_translation(
                    [bbox.x - original_bbox.x, 0., 0.].into(),
                ));
            }
            XConstraint::Right(right) => {
                bbox.x = parent_bbox.x + parent_bbox.width - (right + bbox.width);
                transforms.push(Mat4::from_translation(
                    [bbox.x - original_bbox.x, 0., 0.].into(),
                ));
            }
            XConstraint::LeftAndRight { left, right } => {
                bbox.width = bbox.width - (left + right);
                bbox.x += left;
                // TODO: Scale
                transforms.push(Mat4::from_translation(
                    [bbox.x - original_bbox.x, 0., 0.].into(),
                ))
            }
            XConstraint::Center(rightward_from_center) => {
                bbox.x = parent_bbox.x_center() + rightward_from_center;
                transforms.push(Mat4::from_translation(
                    [bbox.x - original_bbox.x, 0., 0.].into(),
                ))
            }
            XConstraint::Scale => {
                // TODO: Scale
                bbox.width = parent_bbox.width;
                transforms.push(Mat4::from_scale([1., 1., 1.].into()))
            }
        };

        node.children()
            .into_iter()
            .for_each(|child| layout_recursively(&child, bbox, transforms));
    };
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
    let mut normalize_svg = Mat4::IDENTITY;
    guppies::render_loop(move |event, gpu_redraw| {
        match event {
            guppies::winit::event::Event::WindowEvent { event, .. } => match event {
                guppies::winit::event::WindowEvent::Resized(p) => {
                    normalize_svg = get_svg_normalization(
                        *p,
                        Rect {
                            position: Vec2::new(12., 14.),
                            size: Vec2::new(33., 24.),
                        },
                        Constraint {
                            x: XConstraint::Scale,
                            y: YConstraint::Scale,
                        },
                    );
                }
                _ => {}
            },
            _ => {}
        }
        gpu_redraw.update_triangles(svg_set.get_combined_geometries().triangles, 0);
        gpu_redraw.update_texture(
            [cast_slice(&[
                get_normalization(),
                Mat4::ZERO,
                normalize_svg,
                Mat4::IDENTITY,
                Mat4::IDENTITY,
            ])]
            .concat(),
        );
    });
}
