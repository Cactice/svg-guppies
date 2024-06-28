use experiment::responsive::constraint::YConstraint;
use experiment::responsive::layout_machine::ConstraintMap;
use experiment::serde_json;
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

    let mut list_1 = duplicate(&mut layout_machine, container_name.clone(), 1);
    let mut list_2 = duplicate(&mut layout_machine, container_name, 3);
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

fn duplicate(
    layout_machine: &mut LayoutMachine,
    container_name: String,
    index: u32,
) -> salvage::svg_set::SvgSet {
    let container_name_with_suffix = container_name.clone() + " " + &index.to_string();
    let transform_id = (layout_machine.layouts.len() + 1).try_into().unwrap();
    let list = use_svg(
        include_str!("../V2.svg").to_string(),
        |node, mut pass_down| {
            layout_machine.add_node(&node, &mut pass_down, Some(&index.to_string()));
        },
        Some((
            "ListItem #transform #layout #component".to_string(),
            container_name_with_suffix.clone(),
        )),
        Some(transform_id),
    );

    let mut layout = layout_machine
        .id_to_layout
        .get(&container_name)
        .cloned()
        .expect(&container_name);
    layout.constraint.y = match layout.constraint.y {
        YConstraint::Top(y) => YConstraint::Top(y + 80.0 * index as f32),
        y => y,
    };
    layout_machine
        .id_to_layout
        .insert(container_name_with_suffix, layout);
    list
}

#[mobile_entry_point]
pub fn mobile_main() {
    main()
}
