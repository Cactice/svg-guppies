use salvage::{svg_set::SvgSet, usvg::Node};

use crate::{
    responsive::{constraint::YConstraint, layout_machine::LayoutMachine},
    svg_init::{get_default_init_callback, PassDown},
};

pub fn use_svg<C: FnMut(&Node, &mut PassDown)>(
    xml: String,
    mut callback: C,
    component_and_destination: Option<(String, String)>,
    transform_count: Option<u32>,
) -> SvgSet {
    let initial_pass_down = PassDown {
        is_include: component_and_destination.is_none(),
        parent: component_and_destination
            .clone()
            .map(|component_and_destination| component_and_destination.1),
        ..Default::default()
    };
    let transform_count = transform_count.unwrap_or(1);
    let mut default_callback =
        get_default_init_callback(transform_count, component_and_destination.map(|x| x.0));
    SvgSet::new(xml.to_string(), initial_pass_down, |node, passdown| {
        let (geometry, mut passdown) = default_callback(node.clone(), passdown);
        callback(&node, &mut passdown);
        (geometry, passdown)
    })
}

pub fn use_duplicate(
    xml: String,
    layout_machine: &mut LayoutMachine,
    component_name: String,
    container_name: String,
    index: u32,
    offset: f32,
) -> salvage::svg_set::SvgSet {
    let container_name_with_suffix = container_name.clone() + " " + &index.to_string();
    let transform_id = (layout_machine.layouts.len() + 1).try_into().unwrap();
    let list = use_svg(
        xml,
        |node, mut pass_down| {
            layout_machine.add_node(&node, &mut pass_down, Some(&index.to_string()));
        },
        Some((component_name, container_name_with_suffix.clone())),
        Some(transform_id),
    );

    let mut layout = layout_machine
        .id_to_layout
        .get(&container_name)
        .cloned()
        .expect(&container_name);
    layout.constraint.y = match layout.constraint.y {
        YConstraint::Top(y) => YConstraint::Top(y + offset * index as f32),
        y => y,
    };
    layout_machine
        .id_to_layout
        .insert(container_name_with_suffix, layout);
    list
}
