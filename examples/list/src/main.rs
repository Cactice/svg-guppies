use experiment::responsive::constraint::YConstraint;
use experiment::responsive::layout_machine::ConstraintMap;
use experiment::serde_json;
use experiment::uses::use_duplicate;
use experiment::{responsive::layout_machine::LayoutMachine, uses::use_svg};
use guppies::bytemuck::cast_slice;
use guppies::{GpuRedraw, Guppy};
use mobile_entry_point::mobile_entry_point;

pub fn main() {
    let mut layout_machine = LayoutMachine::default();
    let json = include_str!("constraints.json");
    layout_machine.constraint_map = serde_json::from_str::<ConstraintMap>(json).unwrap();

    let svg_set = use_svg(
        include_str!("../V2.svg").to_string(),
        |node, mut pass_down| {
            layout_machine.add_node(&node, &mut pass_down, None);
        },
        None,
        None,
    );
    let container_name = "ComponentBox #transform #layout".to_owned();

    let component_name = "ListItem #transform #layout #component".to_string();
    let xml = &include_str!("../V2.svg");
    let mut list_1 = use_duplicate(
        xml.to_string(),
        &mut layout_machine,
        component_name.clone(),
        container_name.clone(),
        0,
        70.0,
    );
    let mut list_2 = use_duplicate(
        xml.to_string(),
        &mut layout_machine,
        component_name,
        container_name.clone(),
        1,
        70.0,
    );
    list_1.update_text("word #dynamicText #transform #layout", "abb");
    list_2.update_text("word #dynamicText #transform #layout", "abbbbbbbabfdkj");

    let mut guppy = Guppy::new([GpuRedraw::default()]);

    guppy.register(move |event, gpu_redraws| {
        layout_machine.event_handler(event);
        gpu_redraws[0].update_texture(cast_slice(&layout_machine.transforms.clone()).to_vec());
        gpu_redraws[0].update_triangles(
            list_1
                .get_combined_geometries()
                .extend(&list_2.get_combined_geometries())
                .extend(&svg_set.get_combined_geometries())
                .triangles,
            0,
        );
    });

    guppy.start();
}

#[mobile_entry_point]
pub fn mobile_main() {
    main()
}
