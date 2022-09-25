use bytemuck::cast_slice;
use concept::{scroll::ScrollState, uses::use_svg};
use guppies::glam::Mat4;

pub fn main() {
    let svg_set = use_svg(include_str!("../Menu.svg"), |_, _| {});
    let mut scroll_state = ScrollState::new_from_svg_set(&svg_set);
    guppies::render_loop(move |event, gpu_redraw| {
        scroll_state.event_handler(event);
        gpu_redraw.update_triangles(svg_set.get_combined_geometries().triangles, 0);
        gpu_redraw
            .update_texture([cast_slice(&[scroll_state.transform, Mat4::default()])].concat());
    });
}
