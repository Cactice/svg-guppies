use bytemuck::cast_slice;
use concept::{
    constraint::{Clickable, ClickableBbox, Constraint, Layout},
    scroll::ScrollState,
    svg_init::{regex::Regex, CLICKABLE_REGEX, LAYOUT_REGEX, TRANSFORM_REGEX},
    uses::use_svg,
};
use guppies::{
    glam::{Mat4, Vec4},
    primitives::Rect,
    winit::{
        dpi::PhysicalSize,
        event::{ElementState, WindowEvent},
    },
};
use mobile_entry_point::mobile_entry_point;
use salvage::usvg::{self, NodeExt, PathBbox};
use std::vec;

fn bbox_to_mat4(bbox: PathBbox) -> Mat4 {
    Mat4::from_scale_rotation_translation(
        [bbox.width() as f32, bbox.height() as f32, 1.].into(),
        Default::default(),
        [bbox.x() as f32, bbox.y() as f32, 0.].into(),
    )
}

fn get_layout(node: &usvg::Node) -> Option<Layout> {
    let layout_regex = Regex::new(LAYOUT_REGEX).unwrap();
    let id = node.id();
    if layout_regex.is_match(&id) {
        return Some(Layout::new(node));
    }
    None
}

pub fn main() {
    let mut layouts = Vec::new();
    let mut display_mat4 = Mat4::IDENTITY;
    let mut svg_mat4 = Mat4::IDENTITY;
    let mut clickables = Vec::new();
    let clickable_regex = Regex::new(CLICKABLE_REGEX).unwrap();
    let mut scroll_state = ScrollState::default();

    let svg_set = use_svg(
        include_str!("../MenuBar.svg").to_string(),
        |node, _pass_down| {
            let some_layout = get_layout(&node);
            if let Some(layout) = some_layout {
                layouts.push(layout);
            };

            let id = node.id().to_string();
            if clickable_regex.is_match(&id) {
                let clickable: Clickable = if let Some(layout) = some_layout {
                    Clickable {
                        bbox: ClickableBbox::Layout(layout),
                        id,
                    }
                } else {
                    let bbox_mat4 = bbox_to_mat4(node.calculate_bbox().unwrap());
                    Clickable {
                        bbox: ClickableBbox::Bbox(bbox_mat4),
                        id,
                    }
                };
                clickables.push(clickable)
            }
        },
    );

    guppies::render_loop::<1, _, _>(move |event, gpu_redraw| {
        scroll_state.event_handler(event);
        if let guppies::winit::event::Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(p) => {
                    let mut transforms = vec![Mat4::IDENTITY, Mat4::IDENTITY];
                    transforms.append(
                        &mut layouts
                            .iter()
                            .map(|layout| layout.to_mat4_new(p, &svg_set))
                            .collect(),
                    );
                    gpu_redraw[0].update_texture([cast_slice(&transforms[..])].concat());
                    gpu_redraw[0].update_triangles(svg_set.get_combined_geometries().triangles, 0);
                }

                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    ..
                } => {
                    let click = Vec4::from((scroll_state.mouse_position, 1., 1.));
                    let clicked_ids = clickables
                        .iter()
                        .filter_map(|clickable| {
                            if clickable
                                .bbox
                                .click_detection(click, display_mat4, svg_mat4)
                            {
                                Some(clickable.id.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<String>>();
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
