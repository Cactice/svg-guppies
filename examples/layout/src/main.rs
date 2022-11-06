mod call_back;
mod rect;
use bytemuck::cast_slice;
use call_back::{get_my_init_callback, get_screen_size, get_svg_size};
use guppies::glam::Mat4;
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
        let bbox_mat4 = Mat4::from_translation(
            [
                (bbox.x() + bbox.width() / 2.) as f32,
                (bbox.y() + bbox.height() / 2.) as f32,
                0.0 as f32,
            ]
            .into(),
        )
        .inverse();
        // let constraint_x = get_x_constraint(&node.id());
        // match constraint_x {
        //     XConstraint::Left(left) => {
        //         let align_left = parent_mat4.inverse()
        //             * Mat4::from_translation((-0.5, 0., 0.).into())
        //             * parent_mat4;
        //         let constraint_translation = Mat4::from_translation((left, 0., 0.).into());
        //         this_mat4 = parent_mat4 * constraint_translation
        //     }
        //     XConstraint::Right(right) => {
        //         let align_right = Mat4::from_translation((0.5, 0., 0.).into());
        //         let constraint_translation = Mat4::from_translation((right, 0., 0.).into());
        //         this_mat4 = constraint_translation * display_mat4 * parent_mat4
        //     }
        //     XConstraint::LeftAndRight { left, right } => {
        //         let (bbox_scale, _, _) = bbox_mat4.to_scale_rotation_translation();
        //         let (parent_scale, _, _) = parent_mat4.to_scale_rotation_translation();
        //         let constraint_scale = Mat4::from_scale(
        //             (parent_scale.x + left + right / parent_scale.x, 1., 1.).into(),
        //         );
        //         let constraint_translation = Mat4::from_translation((left, 0., 0.).into());
        //         this_mat4 =
        //             constraint_translation * constraint_translation * constraint_scale * parent_mat4
        //     }
        //     XConstraint::Center(rightward_from_center) => {
        //         let constraint_translation =
        //             Mat4::from_translation((rightward_from_center, 0., 0.).into());
        //         this_mat4 = display_mat4 * constraint_translation * parent_mat4
        //     }
        //     XConstraint::Scale => this_mat4 = parent_mat4,
        // }
        if node.id().contains("#transform") {
            dbg!(node.id());
            children_transforms.insert(
                0,
                Mat4::from_scale([2., 2., 1.].into()) * display_mat4.inverse() * bbox_mat4,
            );
        }
    } else {
    };

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
                    // let svg_normalization = Mat4::from_translation([-0.5, -0.5, 1.0].into())
                    //     * get_svg_normalization(svg_set.bbox);

                    let mut transforms =
                        layout_recursively(svg_mat4, display_mat4, svg_set.root.clone(), svg_mat4);
                    let mut answer_transforms = vec![Mat4::IDENTITY, Mat4::IDENTITY];
                    transforms.iter().enumerate().for_each(|(i, transform)| {
                        dbg!(i, transform.to_scale_rotation_translation());
                    });
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
