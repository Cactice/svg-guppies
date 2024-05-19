use salvage::{geometry::Geometry, svg_set::SvgSet, usvg::Node};

use crate::svg_init::{get_default_init_callback, PassDown};

pub fn use_svg<C: FnMut(&Node, &mut PassDown)>(
    xml: String,
    mut callback: C,
    component_and_destination: Option<(String, Option<String>)>,
    transform_count: Option<u32>,
) -> SvgSet {
    let initial_pass_down = PassDown {
        is_include: component_and_destination.is_none(),
        parent: component_and_destination
            .clone()
            .map(|component_and_destination| component_and_destination.1)
            .flatten(),
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
