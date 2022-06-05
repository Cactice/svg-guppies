mod convert_path;
mod fill;
pub mod geometry;
mod stroke;
use geometry::{Callback, DrawPrimitives, Geometry, GeometrySet, IndicesPriority, Rect};
pub use glam;
use glam::{DMat4, DVec2, Vec2, Vec4};
use lyon::lyon_tessellation::{FillVertex, StrokeVertex, VertexBuffers};
use roxmltree::{Document, NodeId};
use std::{collections::HashMap, ops::Range, sync::Arc};
pub use usvg;
use usvg::{fontdb::Source, Node, NodeKind, Path, Tree};

struct TransformVariable {
    transform: DMat4,
    transform_index: u16,
}

#[derive(Clone, Debug)]
struct SvgSet {
    geometry_set: GeometrySet,
    root: Node,
    id_map: HashMap<String, NodeId>,
}

fn recursive_svg(
    node: usvg::Node,
    parent_priority: IndicesPriority,
    callback: &mut Callback,
    geometry_set: &mut GeometrySet,
    mut ids: Vec<String>,
) {
    let priority = parent_priority.max(callback.process_events(&node));
    let node_ref = &node.borrow();
    let id = NodeKind::id(node_ref);
    if id != "" {
        ids.push(id.to_string());
    }

    if let usvg::NodeKind::Path(ref p) = *node.borrow() {
        let geometry = Geometry::new(&p, geometry_set.get_vertices_len(priority), ids.to_vec());
        geometry_set.push_with_priority(geometry, priority);
    }
    for child in node.children() {
        recursive_svg(child, priority, callback, geometry_set, ids.clone());
    }
}

pub fn init(mut callback: Callback) -> (DrawPrimitives, Rect) {
    // Parse and tessellate the geometry

    let contents = include_bytes!("../fallback_font/Roboto-Medium.ttf");
    let mut opt = usvg::Options::default();
    opt.fontdb
        .load_font_source(Source::Binary(Arc::new(contents.as_ref())));
    opt.font_family = "Roboto Medium".to_string();
    opt.keep_named_groups = true;

    let mut geometry_set = GeometrySet::default();
    let tree = roxmltree::Document::parse(include_str!("../../svg/life_text.svg")).unwrap();
    let rtree = Tree::from_xmltree(&tree, &opt.to_ref()).unwrap();
    let id_map = tree
        .descendants()
        .fold(HashMap::<String, NodeId>::new(), |mut acc, curr| {
            if let Some(attribute_id) = tree.root().attribute("id") {
                acc.insert(attribute_id.to_string(), curr.id());
            }
            acc
        });
    recursive_svg(
        rtree.root(),
        IndicesPriority::Fixed,
        &mut callback,
        &mut geometry_set,
        vec![],
    );

    let view_box = rtree.svg_node().view_box;
    let rect: Rect = (
        Vec2::new(view_box.rect.x() as f32, view_box.rect.y() as f32),
        Vec2::new(view_box.rect.width() as f32, view_box.rect.height() as f32),
    );

    (
        (geometry_set.get_vertices(), geometry_set.get_indices()),
        rect,
    )
}
