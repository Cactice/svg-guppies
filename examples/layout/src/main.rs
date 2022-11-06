mod call_back;
mod rect;
use bytemuck::cast_slice;
use call_back::{get_my_init_callback, get_screen_size, get_svg_size, get_x_constraint};
use guppies::glam::Mat4;
use rect::XConstraint;
use salvage::{
    callback::IndicesPriority,
    svg_set::SvgSet,
    usvg::{self, NodeExt, PathBbox},
};
use std::vec;

#[derive(Clone, Default)]
pub struct MyPassDown {
    pub indices_priority: IndicesPriority,
    pub transform_id: u32,
    pub bbox: Option<PathBbox>,
}

fn layout_recursively(
    svg_mat4: Mat4,
    display_mat4: Mat4,
    node: usvg::Node,
    parent_mat4: Mat4,
) -> Vec<Mat4> {
    let mut children_transforms: Vec<Mat4> = node
        .children()
        .into_iter()
        .flat_map(|child| layout_recursively(svg_mat4, display_mat4, child, parent_mat4))
        .collect();

    let bbox = node.calculate_bbox();
    if let Some(bbox) = bbox {
        let fill_mat4 = Mat4::from_scale(
            [
                display_mat4.to_scale_rotation_translation().0.x
                    / svg_mat4.to_scale_rotation_translation().0.x,
                display_mat4.to_scale_rotation_translation().0.y
                    / svg_mat4.to_scale_rotation_translation().0.y,
                1.,
            ]
            .into(),
        );
        let left_align_mat4 =
            Mat4::from_translation([bbox.x() as f32, bbox.y() as f32, 0.0 as f32].into()).inverse();
        let right_align_mat4 = Mat4::from_translation(
            [
                (bbox.x() + bbox.width()) as f32,
                (bbox.y() + bbox.height()) as f32,
                0.0 as f32,
            ]
            .into(),
        )
        .inverse();
        let center_mat4 = Mat4::from_translation(
            [
                (bbox.x() + bbox.width() / 2.) as f32,
                (bbox.y() + bbox.height() / 2.) as f32,
                0.0 as f32,
            ]
            .into(),
        )
        .inverse();

        let constraint_x = get_x_constraint(&node.id());
        let pre_translation_mat4;
        let post_translation_mat4;
        let scale_mat4;
        match constraint_x {
            XConstraint::Left(left) => {
                pre_translation_mat4 =
                    left_align_mat4 * Mat4::from_translation([left, 0., 0.].into());
                post_translation_mat4 = Mat4::from_translation([-1.0, 0., 0.].into());
                scale_mat4 = Mat4::IDENTITY;
            }
            XConstraint::Right(right) => {
                pre_translation_mat4 =
                    right_align_mat4 * Mat4::from_translation([right, 0., 0.].into());
                post_translation_mat4 = Mat4::from_translation([1.0, 0., 0.].into());
                scale_mat4 = Mat4::IDENTITY;
            }
            XConstraint::Center(rightward_from_center) => {
                pre_translation_mat4 =
                    center_mat4 * Mat4::from_translation([rightward_from_center, 0., 0.].into());
                post_translation_mat4 = Mat4::IDENTITY;
                scale_mat4 = Mat4::IDENTITY;
            }
            XConstraint::LeftAndRight { left, right } => {
                todo!();
            }
            XConstraint::Scale => {
                pre_translation_mat4 = center_mat4;
                post_translation_mat4 = Mat4::IDENTITY;
                scale_mat4 = fill_mat4;
            }
        }
        if node.id().contains("#transform") {
            children_transforms.insert(
                0,
                post_translation_mat4
                    * Mat4::from_scale([2., 2., 1.].into())
                    * display_mat4.inverse()
                    * scale_mat4
                    * pre_translation_mat4,
            );
        }
    }
    return children_transforms;
}

pub fn main() {
    let svg_set = SvgSet::new(
        include_str!("../MenuBar.svg"),
        MyPassDown {
            transform_id: 1,
            ..Default::default()
        },
        get_my_init_callback(),
    );
    guppies::render_loop(move |event, gpu_redraw| {
        match event {
            guppies::winit::event::Event::WindowEvent { event, .. } => match event {
                guppies::winit::event::WindowEvent::Resized(p) => {
                    let display_mat4 = get_screen_size(*p);
                    let svg_mat4 = get_svg_size(svg_set.bbox);

                    let mut transforms =
                        layout_recursively(svg_mat4, display_mat4, svg_set.root.clone(), svg_mat4);
                    let mut answer_transforms = vec![Mat4::IDENTITY, Mat4::IDENTITY];
                    // transforms.iter().enumerate().for_each(|(i, transform)| {
                    // dbg!(i, transform.to_scale_rotation_translation());
                    // });
                    answer_transforms.append(&mut transforms);
                    gpu_redraw.update_texture([cast_slice(&answer_transforms[..])].concat());
                }
                _ => {}
            },
            _ => {}
        }
        gpu_redraw.update_triangles(svg_set.get_combined_geometries().triangles, 0);
        // gpu_redraw.update_texture([cast_slice(&transforms[..])].concat());
    });
}
