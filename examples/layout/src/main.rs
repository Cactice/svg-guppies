use bytemuck::cast_slice;
use concept::{
    responsive::layout::{Layout, LayoutMachine},
    scroll::ScrollState,
    uses::use_svg,
};
use guppies::{
    glam::{Mat4, Vec4},
    winit::event::{ElementState, WindowEvent},
};
use mobile_entry_point::mobile_entry_point;
use std::vec;

pub fn main() {
    let mut layout_machine = LayoutMachine::default();
    let mut scroll_state = ScrollState::default();

    let svg_set = use_svg(
        include_str!("../MenuBar.svg").to_string(),
        |node, _pass_down| {
            layout_machine.add_node(node);
        },
    );

    guppies::render_loop::<1, _, _>(move |event, gpu_redraw| {
        scroll_state.event_handler(event);
        if let guppies::winit::event::Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(p) => {
                    layout_machine.resize(p);
                    let mut transforms = vec![Mat4::IDENTITY, Mat4::IDENTITY];
                    transforms.append(&mut layout_machine.get_transforms());
                    gpu_redraw[0].update_texture([cast_slice(&transforms[..])].concat());
                    gpu_redraw[0].update_triangles(svg_set.get_combined_geometries().triangles, 0);
                }

                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    ..
                } => {
                    layout_machine.click_detection(&scroll_state);
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
