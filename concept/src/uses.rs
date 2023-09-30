use salvage::{callback::PassDown, svg_set::SvgSet, usvg::Node};

use crate::svg_init::get_default_init_callback;

pub fn use_svg<C: FnMut(Node, PassDown)>(xml: String, mut callback: C) -> SvgSet {
    let mut default_callback = get_default_init_callback();
    SvgSet::new(xml.to_string(), PassDown::default(), |node, passdown| {
        callback(node.clone(), passdown);
        default_callback(node, passdown)
    })
}

pub fn use_svgs<C: FnMut(Node, PassDown)>(xml: String, mut callback: C) -> SvgSet {
    let mut default_callback = get_default_init_callback();
    SvgSet::new(xml.to_string(), PassDown::default(), |node, passdown| {
        callback(node.clone(), passdown);
        default_callback(node, passdown)
    })
}
