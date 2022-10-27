mod call_back;
mod rect;
use bytemuck::cast_slice;
use call_back::{get_my_init_callback, get_normalization, get_svg_normalization, get_x_constraint};
use guppies::glam::{Mat4, Vec3};
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
fn layout_recursively(
    display_mat4: Mat4,
    node: &Node,
    parent_mat4: Mat4,
    transforms: &mut Vec<Mat4>,
) {
    let bbox = node.calculate_bbox();
    if let Some(bbox) = bbox {
        let mut bbox = Mat4::from_scale((bbox.x() as f32, bbox.y() as f32, 0.0).into());
        let constraint_x = get_x_constraint(&node.id());
        let mut new_display_mat4 = display_mat4;

        let mat4 = match constraint_x {
            XConstraint::Left(left) => {
                let align_left = Mat4::from_translation((-0.5, 0., 0.).into());
                let constraint_translation = Mat4::from_translation((left, 0., 0.).into());
                new_display_mat4 = Mat4::IDENTITY;
                constraint_translation * display_mat4 * align_left * parent_mat4
            }
            XConstraint::Right(right) => {
                let align_right = Mat4::from_translation((0.5, 0., 0.).into());
                let constraint_translation = Mat4::from_translation((right, 0., 0.).into());
                new_display_mat4 = Mat4::IDENTITY;
                constraint_translation * display_mat4 * align_right * parent_mat4
            }
            XConstraint::LeftAndRight { left, right } => {
                let (bbox_scale, _, _) = bbox.to_scale_rotation_translation();
                let (parent_scale, _, _) = parent_mat4.to_scale_rotation_translation();
                let constraint_scale = Mat4::from_scale(
                    (parent_scale.x + left + right / parent_scale.x, 1., 1.).into(),
                );
                let constraint_translation = Mat4::from_translation((left, 0., 0.).into());
                constraint_translation * constraint_scale * parent_mat4
            }
            XConstraint::Center(rightward_from_center) => {
                let constraint_translation =
                    Mat4::from_translation((rightward_from_center, 0., 0.).into());
                new_display_mat4 = Mat4::IDENTITY;
                constraint_translation * display_mat4 * parent_mat4
            }
            XConstraint::Scale => Mat4::IDENTITY,
        };

        if node.id().contains("#transform") {
            transforms.push(mat4)
        }
        node.children()
            .into_iter()
            .for_each(|child| layout_recursively(new_display_mat4, &child, bbox, transforms));
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
                    normalize_svg = get_normalization() * get_svg_normalization(*p);
                }
                _ => {}
            },
            _ => {}
        }
        gpu_redraw.update_triangles(svg_set.get_combined_geometries().triangles, 0);
        gpu_redraw.update_texture(
            [cast_slice(&[
                normalize_svg,
                Mat4::IDENTITY,
                Mat4::IDENTITY,
                Mat4::IDENTITY,
                Mat4::IDENTITY,
            ])]
            .concat(),
        );
    });
}
