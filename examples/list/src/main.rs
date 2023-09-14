use bytemuck::cast_slice;
use concept::{scroll::ScrollState, uses::use_svg};
use guppies::{glam::Mat4, primitives::Vertex};
use mobile_entry_point::mobile_entry_point;

struct ListItem {
    word: String,
    icon: String,
}

pub fn main() {
    let svg_set = use_svg(
        include_str!("../Left.svg").to_string(),
        |_node, _pass_down| {},
    );

    let mut scroll_state = ScrollState::new_from_svg_set(&svg_set);
    guppies::render_loop::<1, Vertex, _>(move |event, gpu_redraw: _| {
        scroll_state.event_handler(event);
        let transforms = [scroll_state.transform, Mat4::IDENTITY].to_vec();
        if let guppies::winit::event::Event::WindowEvent { event, .. } = event {
            gpu_redraw[0].update_triangles(svg_set.get_combined_geometries().triangles, 0);
            gpu_redraw[0].update_texture([cast_slice(&transforms[..])].concat());
        }
    });
}

#[mobile_entry_point]
pub fn mobile_main() {
    main()
}
