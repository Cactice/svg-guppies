use experiment::responsive::constraint::YConstraint;
use experiment::responsive::layout_machine::ConstraintMap;
use experiment::serde_json;
use experiment::{responsive::layout_machine::LayoutMachine, uses::use_svg};
use guppies::bytemuck::cast_slice;
use guppies::glam::Mat4;
use guppies::{GpuRedraw, Guppy};
use mobile_entry_point::mobile_entry_point;

pub fn main() {
    let mut layout_machine = LayoutMachine::default();
    let json = include_str!("constraints.json");
    layout_machine.constraint_map = serde_json::from_str::<ConstraintMap>(json).unwrap();

    let mut transform_id = 0;
    let svg_set = use_svg(
        include_str!("../V2.svg").to_string(),
        |node, mut pass_down| {
            layout_machine.add_node(&node, &mut pass_down);
            transform_id = transform_id.max(pass_down.transform_id);
        },
        None,
        None,
    );
    let var_name = transform_id.clone();
    let container_name = "ComponentBox #transform #layout".to_owned();
    let mut x = layout_machine
        .id_to_layout
        .get(&container_name)
        .cloned()
        .unwrap();
    x.constraint.y = match x.constraint.y {
        YConstraint::Top(x) => YConstraint::Top(x + 30.0),
        y => y,
    };
    let list = use_svg(
        include_str!("../V2.svg").to_string(),
        |node, mut pass_down| {
            layout_machine.add_node(&node, &mut pass_down);
            transform_id = transform_id.max(pass_down.transform_id);
        },
        Some((
            "ListItem #transform #layout #component".to_string(),
            Some(container_name.clone()),
        )),
        Some(var_name),
    );
    layout_machine
        .id_to_layout
        .insert(container_name.clone(), x);

    let mut guppy = Guppy::new([GpuRedraw::default()]);

    guppy.register(move |event, gpu_redraws| {
        layout_machine.event_handler(event);
        gpu_redraws[0].update_texture(
            [cast_slice(
                &[layout_machine.transforms.clone()].concat()[..],
            )]
            .concat(),
        );
        gpu_redraws[0].update_triangles(
            list.get_combined_geometries()
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
