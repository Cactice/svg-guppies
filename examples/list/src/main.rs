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

    let mut transform_id = 0;
    let svg_set = use_svg(
        include_str!("../V2.svg").to_string(),
        |node, mut pass_down| {
            layout_machine.add_node(&node, &mut pass_down, None);
            transform_id = transform_id.max(pass_down.transform_id);
        },
        None,
        None,
    );
    let container_name = "ComponentBox #transform #layout".to_owned();

    let list = duplicate(
        &mut layout_machine,
        container_name.clone(),
        &mut transform_id,
        1,
    );
    let list_2 = duplicate(&mut layout_machine, container_name, &mut transform_id, 2);

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
    transform_id: &mut u32,
    index: u32,
) -> salvage::svg_set::SvgSet {
    let var_name = transform_id.clone();
    let mut layout = layout_machine
        .id_to_layout
        .get(&container_name)
        .cloned()
        .unwrap();
    let container_name_2 = container_name + " " + &index.to_string();
    layout.constraint.y = match layout.constraint.y {
        YConstraint::Top(y) => YConstraint::Top(y + 80.0 * index as f32),
        y => y,
    };
    let list = use_svg(
        include_str!("../V2.svg").to_string(),
        |node, mut pass_down| {
            layout_machine.add_node(&node, &mut pass_down, Some(&index.to_string()));
            *transform_id = (*transform_id).max(pass_down.transform_id);
        },
        Some((
            "ListItem #transform #layout #component".to_string(),
            Some(container_name_2.clone()),
        )),
        Some(var_name),
    );

    layout_machine
        .id_to_layout
        .insert(container_name_2.clone(), layout);
    list
}

#[mobile_entry_point]
pub fn mobile_main() {
    main()
}
