use salvage::{geometry::Geometry, svg_set::SvgSet, usvg::Node};

use crate::svg_init::{get_default_init_callback, PassDown};

pub fn use_svg<C: FnMut(&Node, &mut PassDown)>(
    xml: String,
    mut callback: C,
    component: Option<String>,
    default_callback_args: Option<u32>,
) -> SvgSet {
    let initial_pass_down = PassDown {
        is_include: component.is_none(),
        ..Default::default()
    };
    let args = default_callback_args.unwrap_or(1);
    let mut default_callback = get_default_init_callback(args, component);
    SvgSet::new(xml.to_string(), initial_pass_down, |node, passdown| {
        let (geometry, mut passdown) = default_callback(node.clone(), passdown);
        callback(&node, &mut passdown);
        (geometry, passdown)
    })
}
