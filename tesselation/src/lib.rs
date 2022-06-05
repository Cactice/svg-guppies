mod convert_path;
mod fill;
pub mod geometry;
mod stroke;
use geometry::{Callback, DrawPrimitives, Geometry, GeometrySet, IndicesPriority, Rect, SvgSet};
pub use glam;
use glam::{DMat4, Vec2};

use roxmltree::NodeId;
use std::{collections::HashMap, sync::Arc};
pub use usvg;
use usvg::{fontdb::Source, Node, NodeKind, Tree};

struct TransformVariable {
    transform: DMat4,
    transform_index: u16,
}

pub fn init(callback: Callback) -> (DrawPrimitives, Rect) {
    // Parse and tessellate the geometry

    let svg_set = SvgSet::new(include_str!("../../svg/life_text.svg"), callback);
    (
        (
            svg_set.geometry_set.get_vertices(),
            svg_set.geometry_set.get_indices(),
        ),
        svg_set.bbox,
    )
}
