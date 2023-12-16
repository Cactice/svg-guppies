use salvage::{svg_set::SvgSet, usvg::Node};

use crate::svg_init::{get_default_init_callback, PassDown};

pub fn use_svg<C: FnMut(&Node, &mut PassDown)>(
    xml: String,
    mut callback: C,
    component: Option<String>,
) -> SvgSet {
    let initial_pass_down = PassDown {
        is_include: component.is_none(),
        ..Default::default()
    };
    let mut default_callback = get_default_init_callback(1, component);
    SvgSet::new(xml.to_string(), initial_pass_down, |node, mut passdown| {
        callback(&node, &mut passdown);
        default_callback(node, passdown)
    })
}
